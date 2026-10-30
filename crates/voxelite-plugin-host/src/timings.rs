use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

/// Aggregate elapsed time for one plugin and label.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Timing {
    /// Number of invocations, including invocations that unwind.
    pub count: u64,
    /// Sum of elapsed times, saturating at [`Duration::MAX`].
    pub total: Duration,
    /// Longest invocation.
    pub max: Duration,
}

/// An invocation exceeded the configured advisory budget.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BudgetExceeded {
    /// Plugin responsible for this invocation.
    pub plugin_id: String,
    /// Operation label.
    pub label: String,
    /// Observed elapsed time.
    pub elapsed: Duration,
    /// Budget in effect for this invocation.
    pub budget: Duration,
}

/// Per-plugin timing recorder with an optional per-invocation advisory budget.
///
/// A budget never interrupts work or faults a plugin. Timing a closure preserves
/// its return value or unwind; panic isolation is the host's responsibility.
#[derive(Debug, Default)]
pub struct Timings {
    entries: BTreeMap<String, BTreeMap<String, Timing>>,
    budget: Option<Duration>,
    notices: Vec<BudgetExceeded>,
}

impl Timings {
    /// Create an empty recorder with an optional budget for every label.
    pub fn new(budget: Option<Duration>) -> Self {
        Self {
            budget,
            ..Self::default()
        }
    }

    /// Change the advisory budget for subsequent invocations.
    pub fn set_budget(&mut self, budget: Option<Duration>) {
        self.budget = budget;
    }

    /// The current per-invocation budget.
    pub fn budget(&self) -> Option<Duration> {
        self.budget
    }

    /// Run and measure an operation, also recording time when it unwinds.
    pub fn time<T>(&mut self, plugin_id: &str, label: &str, operation: impl FnOnce() -> T) -> T {
        let sample = Sample {
            recorder: self,
            plugin_id,
            label,
            start: Instant::now(),
        };
        let result = operation();
        drop(sample);
        result
    }

    /// Look up aggregate measurements for a plugin and label.
    pub fn get(&self, plugin_id: &str, label: &str) -> Option<&Timing> {
        self.entries.get(plugin_id)?.get(label)
    }

    /// Measurements ordered by plugin ID and then label.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str, &Timing)> {
        self.entries.iter().flat_map(|(plugin_id, labels)| {
            labels
                .iter()
                .map(move |(label, timing)| (plugin_id.as_str(), label.as_str(), timing))
        })
    }

    /// Take all pending budget notices, leaving the measurements intact.
    pub fn drain_notices(&mut self) -> Vec<BudgetExceeded> {
        std::mem::take(&mut self.notices)
    }

    pub(crate) fn record(&mut self, plugin_id: &str, label: &str, elapsed: Duration) {
        let timing = self
            .entries
            .entry(plugin_id.to_owned())
            .or_default()
            .entry(label.to_owned())
            .or_default();
        timing.count = timing.count.saturating_add(1);
        timing.total = timing.total.saturating_add(elapsed);
        timing.max = timing.max.max(elapsed);
        if let Some(budget) = self.budget
            && elapsed > budget
        {
            self.notices.push(BudgetExceeded {
                plugin_id: plugin_id.to_owned(),
                label: label.to_owned(),
                elapsed,
                budget,
            });
        }
    }
}

struct Sample<'a> {
    recorder: &'a mut Timings,
    plugin_id: &'a str,
    label: &'a str,
    start: Instant,
}

impl Drop for Sample<'_> {
    fn drop(&mut self) {
        self.recorder
            .record(self.plugin_id, self.label, self.start.elapsed());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aggregates_and_budgets_are_independent_per_plugin_and_label() {
        let mut timings = Timings::new(Some(Duration::from_millis(3)));
        for millis in [2, 3, 5] {
            timings.record("dev.first", "render", Duration::from_millis(millis));
        }
        timings.record("dev.first", "activate", Duration::from_millis(1));
        timings.record("dev.second", "render", Duration::from_millis(2));
        assert_eq!(
            timings.get("dev.first", "render"),
            Some(&Timing {
                count: 3,
                total: Duration::from_millis(10),
                max: Duration::from_millis(5),
            })
        );
        assert_eq!(timings.iter().count(), 3);
        assert_eq!(
            timings.drain_notices(),
            vec![BudgetExceeded {
                plugin_id: "dev.first".into(),
                label: "render".into(),
                elapsed: Duration::from_millis(5),
                budget: Duration::from_millis(3),
            }]
        );
        assert!(timings.drain_notices().is_empty());
        timings.set_budget(None);
        timings.record("dev.first", "render", Duration::MAX);
        assert_eq!(
            timings.get("dev.first", "render").unwrap().total,
            Duration::MAX
        );
        assert!(timings.drain_notices().is_empty());
    }

    #[test]
    fn time_preserves_results_and_unwinds_and_records_both() {
        let mut timings = Timings::default();
        assert_eq!(timings.time("dev.first", "work", || 42), 42);
        let panic = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            timings.time("dev.first", "work", || panic!("timed panic"));
        }));
        assert!(panic.is_err());
        let timing = timings.get("dev.first", "work").unwrap();
        assert_eq!(timing.count, 2);
        assert!(timing.total >= timing.max);
        assert!(timings.drain_notices().is_empty());
    }
}
