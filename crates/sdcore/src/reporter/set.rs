use super::ReportEvent;

pub trait Reporter {
    fn report(&mut self, event: &ReportEvent<'_>);
}

pub struct ReporterSet {
    reporters: Vec<Box<dyn Reporter>>,
}

impl ReporterSet {
    pub fn new(reporters: Vec<Box<dyn Reporter>>) -> Self {
        Self { reporters }
    }

    pub fn report(&mut self, event: &ReportEvent<'_>) {
        for reporter in &mut self.reporters {
            reporter.report(event);
        }
    }
}
