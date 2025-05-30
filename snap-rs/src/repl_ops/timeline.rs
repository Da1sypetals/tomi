use super::memsnap::MemSnap;
use plotters::{
    chart::{ChartBuilder, LabelAreaPosition},
    prelude::{BitMapBackend, IntoDrawingArea, SVGBackend},
    series::LineSeries,
    style::{GREEN, WHITE},
};
use std::collections::BTreeMap;

pub struct Timeline {
    pub timeline: Vec<(u32, u64)>,
    pub max_time: u32,
    pub max_alloc: u64,
}

impl MemSnap {
    /// Returns (timestamp -> memory occupied) mapping as a Vec
    pub fn build_timeline(&mut self) {
        if self.timeline.is_none() {
            let mut timeline = BTreeMap::new();
            let mut max_time = 0;
            let mut max_alloc = 0;

            for alloc in &self.allocations {
                for (&timestamp, &offset) in alloc.timesteps.iter().zip(alloc.offsets.iter()) {
                    let mem = offset + alloc.size;
                    max_time = max_time.max(timestamp);
                    max_alloc = max_alloc.max(mem);
                    // if timeline.get(timestamp) 1) has no value, or 2) has value smaller than mem, set to mem
                    timeline
                        .entry(timestamp)
                        // if has value, AND is applied, take max
                        .and_modify(|current_mem| *current_mem = mem.max(*current_mem))
                        // if has NO value, OR is applied, insert
                        .or_insert(mem);
                }
            }

            self.timeline = Some(Timeline {
                timeline: timeline.into_iter().collect(),
                max_time,
                max_alloc,
            });
        }
    }

    pub fn plot_timeline(&mut self, path: &str) -> anyhow::Result<()> {
        self.build_timeline();

        match &self.timeline {
            Some(timeline) => {
                let root_area = SVGBackend::new(path, (3000, 800)).into_drawing_area();
                root_area.fill(&WHITE)?;

                let mut ctx = ChartBuilder::on(&root_area)
                    .set_label_area_size(LabelAreaPosition::Left, 24)
                    .set_label_area_size(LabelAreaPosition::Bottom, 24)
                    .caption("Memory Trace Timeline", ("sans-serif", 40))
                    .build_cartesian_2d(0..timeline.max_time, 0..timeline.max_alloc)?;

                ctx.configure_mesh().draw()?;

                ctx.draw_series(LineSeries::new(timeline.timeline.clone(), &GREEN))?;

                Ok(())
            }
            None => Err(anyhow::anyhow!("Timeline not built!")),
        }
    }
}
