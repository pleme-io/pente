//! Convergence: `(desired, observed) -> actions`.
//!
//! A reconciler, not a re-applier. The distinction is the whole point and it is
//! easy to lose: a re-applier writes everything every tick and can never tell
//! you whether it changed anything; a reconciler computes the *gap* and
//! therefore has a fixpoint. The property that separates them is
//! [`Plan::is_converged`] — a converged system plans **nothing**.
//!
//! ## Why this is the shape
//!
//! The fleet's own doctrine (the Viggy Method) types any `(desired, observed,
//! action)` concern as a controller. This module is the theme domain's
//! instance, and it is deliberately generic in structure so the desktop's
//! reconciler is the same shape with a different action set rather than a
//! second invention.
//!
//! ## The honest limits, stated rather than discovered
//!
//! - **Observation is of ARTIFACTS, not of consumers.** This module knows that
//!   a file on disk differs from what the generator emits. It does *not* know
//!   whether a running ghostty has re-read its config. Convergence here means
//!   "the artifacts are correct", and an action whose effect is not observable
//!   is typed as such ([`Effect::Blind`]) so it can never be counted as
//!   converged. That is the honest floor, not a gap to paper over.
//! - **Bounded by construction.** The plan is at most one action per artifact,
//!   and the artifact set is `themes × targets`, a compile-time constant. There
//!   is no unbounded work and therefore no tick that can fail to terminate —
//!   which is the first requirement in the fleet's reconciler-liveness rules.

use std::path::{Path, PathBuf};

use pente_spec::FleetTheme;

use crate::Target;

/// Whether an action's effect can be READ BACK.
///
/// A reconciler can only converge on what it can observe. Typing this stops the
/// familiar lie where a fire-and-forget action is reported as reconciled
/// because it was *dispatched*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    /// The effect is observable, so the next tick will confirm it. Convergence
    /// is meaningful.
    Observable,
    /// The effect cannot be read back. The action may be fired, but it must
    /// never be treated as reaching a state — see [`Plan::is_converged`].
    Blind,
}

/// A single unit of work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    /// The artifact is absent or stale; write it.
    WriteArtifact {
        theme: FleetTheme,
        target: Target,
        path: PathBuf,
        /// Why, in the operator's words — surfaced in logs so a tick explains
        /// itself rather than merely reporting a count.
        reason: Drift,
    },
    /// A consumer must be told to re-read. Blind by nature: nothing here can
    /// verify that a running process complied.
    NotifyConsumer { target: Target },
}

impl Action {
    #[must_use]
    pub const fn effect(&self) -> Effect {
        match self {
            Self::WriteArtifact { .. } => Effect::Observable,
            Self::NotifyConsumer { .. } => Effect::Blind,
        }
    }
}

/// Why an artifact needs writing. Distinguished because "absent" and "wrong"
/// are different operational situations, and collapsing them into "changed"
/// loses the one fact an operator wants first.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drift {
    Absent,
    Stale,
}

/// What the world should look like.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Desired {
    /// The selected theme. Every artifact for every theme is still rendered —
    /// selection decides what CONSUMERS read, not what exists — so switching
    /// themes never requires a regeneration step.
    pub theme: FleetTheme,
    /// Where artifacts live.
    pub root: PathBuf,
}

/// The result of a tick.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Plan {
    pub actions: Vec<Action>,
    /// How many artifacts were examined. Carried so a plan that found nothing
    /// because it LOOKED at nothing is distinguishable from a converged one —
    /// the vacuous-green failure mode.
    pub examined: usize,
}

impl Plan {
    /// Converged means: nothing to do, **and** something was actually examined.
    ///
    /// The second clause is not pedantry. A discovery bug that examines zero
    /// artifacts produces an empty action list, and without the denominator
    /// that is indistinguishable from a healthy system.
    #[must_use]
    pub fn is_converged(&self) -> bool {
        self.actions.is_empty() && self.examined > 0
    }

    /// Actions whose effect can be confirmed next tick.
    #[must_use]
    pub fn observable(&self) -> impl Iterator<Item = &Action> {
        self.actions
            .iter()
            .filter(|a| a.effect() == Effect::Observable)
    }
}

/// Read the world, compare it to intent, and say what to do about it.
///
/// Pure apart from the reads: no writes happen here, so a plan can be shown to
/// an operator before anything is touched. That separation is what makes a
/// dry-run the default rather than a flag.
///
/// # Errors
/// Returns a message if an authored theme fails to compile or validate — a
/// broken source is not drift, and must not be reported as such.
pub fn plan(desired: &Desired) -> Result<Plan, String> {
    let mut actions = Vec::new();
    let mut examined = 0usize;

    for theme in FleetTheme::ALL {
        let t = theme
            .load()
            .map_err(|e| format!("theme {theme:?} failed to compile: {e}"))?;
        t.validate()
            .map_err(|e| format!("theme {theme:?} failed to validate: {e}"))?;

        for target in Target::ALL {
            let path = desired.root.join(target.file_name(theme));
            let want = target.render(theme, &t);
            examined += 1;

            let drift = match std::fs::read_to_string(&path) {
                Err(_) => Some(Drift::Absent),
                Ok(got) if got != want => Some(Drift::Stale),
                Ok(_) => None,
            };
            if let Some(reason) = drift {
                actions.push(Action::WriteArtifact {
                    theme,
                    target,
                    path,
                    reason,
                });
            }
        }
    }

    // A consumer is notified only when ITS artifact for the SELECTED theme
    // moved. Notifying on every tick would be the re-applier behaviour this
    // module exists to avoid, and notifying for an unselected theme would wake
    // consumers over a file they do not read.
    let mut notify: Vec<Target> = actions
        .iter()
        .filter_map(|a| match a {
            Action::WriteArtifact { theme, target, .. } if *theme == desired.theme => Some(*target),
            _ => None,
        })
        .collect();
    notify.dedup();
    actions.extend(
        notify
            .into_iter()
            .map(|target| Action::NotifyConsumer { target }),
    );

    Ok(Plan { actions, examined })
}

/// Apply the observable half of a plan.
///
/// Blind actions are returned rather than performed: this crate can write a
/// file, but it has no business deciding how a consumer is signalled. The
/// caller owns that, and gets them back explicitly so they cannot be silently
/// dropped.
///
/// # Errors
/// Returns the first write failure.
pub fn apply(desired: &Desired, plan: &Plan) -> Result<Vec<Action>, String> {
    let mut deferred = Vec::new();
    for action in &plan.actions {
        match action {
            Action::WriteArtifact {
                theme,
                target,
                path,
                ..
            } => {
                let t = theme.load().map_err(|e| format!("theme {theme:?}: {e}"))?;
                if let Some(parent) = Path::new(path).parent() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("{}: {e}", parent.display()))?;
                }
                std::fs::write(path, target.render(*theme, &t))
                    .map_err(|e| format!("{}: {e}", path.display()))?;
            }
            blind @ Action::NotifyConsumer { .. } => deferred.push(blind.clone()),
        }
    }
    let _ = desired;
    Ok(deferred)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("pente-reconcile-{name}"));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn desired(root: PathBuf) -> Desired {
        Desired {
            theme: FleetTheme::Nord,
            root,
        }
    }

    /// An empty tree needs every artifact, and each for the right reason.
    #[test]
    fn an_empty_root_plans_every_artifact_as_absent() {
        let d = desired(tmp("empty"));
        let p = plan(&d).unwrap();
        assert_eq!(p.examined, FleetTheme::ALL.len() * Target::ALL.len());
        let writes: Vec<_> = p.observable().collect();
        assert_eq!(writes.len(), p.examined);
        assert!(p.actions.iter().all(|a| !matches!(
            a,
            Action::WriteArtifact {
                reason: Drift::Stale,
                ..
            }
        )));
        assert!(!p.is_converged());
    }

    /// ★ THE PROPERTY THAT MAKES THIS A RECONCILER: applying a plan reaches a
    /// fixpoint. A re-applier would plan the same work forever.
    #[test]
    fn apply_then_replan_converges_to_nothing() {
        let d = desired(tmp("fixpoint"));
        let first = plan(&d).unwrap();
        assert!(!first.actions.is_empty());
        let deferred = apply(&d, &first).unwrap();

        let second = plan(&d).unwrap();
        assert!(
            second.is_converged(),
            "expected convergence, still planned: {:?}",
            second.actions
        );
        assert_eq!(second.examined, first.examined);

        // The blind half is handed back, never silently dropped.
        assert!(!deferred.is_empty());
        assert!(deferred.iter().all(|a| a.effect() == Effect::Blind));
    }

    /// One drifted byte plans exactly one write, for exactly that artifact.
    #[test]
    fn a_single_drifted_artifact_plans_exactly_one_write() {
        let d = desired(tmp("drift"));
        apply(&d, &plan(&d).unwrap()).unwrap();

        let victim = d
            .root
            .join(Target::NixBase16.file_name(FleetTheme::Borealis));
        std::fs::write(&victim, "corrupted").unwrap();

        let p = plan(&d).unwrap();
        let writes: Vec<_> = p.observable().collect();
        assert_eq!(writes.len(), 1, "expected one write, got {writes:?}");
        assert!(matches!(
            writes[0],
            Action::WriteArtifact {
                theme: FleetTheme::Borealis,
                target: Target::NixBase16,
                reason: Drift::Stale,
                ..
            }
        ));
        // Borealis is not the selected theme, so no consumer is woken.
        assert!(!p
            .actions
            .iter()
            .any(|a| matches!(a, Action::NotifyConsumer { .. })));
    }

    /// A consumer is notified only when the SELECTED theme's artifact moves.
    #[test]
    fn only_the_selected_theme_wakes_consumers() {
        let d = desired(tmp("notify"));
        apply(&d, &plan(&d).unwrap()).unwrap();
        std::fs::write(d.root.join(Target::Css.file_name(FleetTheme::Nord)), "x").unwrap();

        let p = plan(&d).unwrap();
        let notifies: Vec<_> = p
            .actions
            .iter()
            .filter(|a| matches!(a, Action::NotifyConsumer { .. }))
            .collect();
        assert_eq!(notifies.len(), 1);
        assert_eq!(
            notifies[0],
            &Action::NotifyConsumer {
                target: Target::Css
            }
        );
    }

    /// ★ Vacuous convergence is unreachable: an empty plan that examined
    /// nothing is NOT converged. This is the failure mode where a discovery bug
    /// reports a healthy system.
    #[test]
    fn an_empty_plan_that_examined_nothing_is_not_converged() {
        let vacuous = Plan {
            actions: vec![],
            examined: 0,
        };
        assert!(!vacuous.is_converged());
        let real = Plan {
            actions: vec![],
            examined: 15,
        };
        assert!(real.is_converged());
    }

    /// The tick is bounded: at most one write per artifact, and the artifact
    /// set is a compile-time constant. There is no plan that grows without limit.
    #[test]
    fn the_plan_is_bounded_by_the_artifact_matrix() {
        let d = desired(tmp("bounded"));
        let p = plan(&d).unwrap();
        let max = FleetTheme::ALL.len() * Target::ALL.len();
        assert!(p.observable().count() <= max);
        assert_eq!(p.examined, max);
    }

    /// Planning never writes. A dry run is the default, not a flag.
    #[test]
    fn planning_does_not_touch_the_filesystem() {
        let d = desired(tmp("readonly"));
        let _ = plan(&d).unwrap();
        assert_eq!(std::fs::read_dir(&d.root).unwrap().count(), 0);
    }
}
