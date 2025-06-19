use std::{thread, time::Duration};

use anyhow::Result;
use symbol_waterfall::Waterfall;

fn main() -> Result<()> {
    let mut waterfall = Waterfall::new()?;

    loop {
        waterfall.step()?;
        waterfall.render()?;

        thread::sleep(Duration::from_millis(50));
    }
}
