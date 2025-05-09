use std::io::{self, Stdout, Write};

use anyhow::Result;
use crossterm::{
    cursor,
    style::{self, Stylize},
    terminal::{self, Clear, ClearType},
    QueueableCommand,
};

use crossterm::style::{Attribute, Color};
use rand::Rng;

const SYMBOLS: &str = "ﾊﾐﾋｰｳｼﾅﾓﾆｻﾜﾂｵﾘｱﾎﾃﾏｹﾒｴｶｷﾑﾕﾗｾﾈｽﾀﾇﾍｦｲｸｺｿﾁﾄﾉﾌﾔﾖﾙﾚﾛﾝ012345789Z:.\"=*+-<>¦╌ç";
// range of how long it takes for a rune to start fading
const LIFETIME: (u16, u16) = (4, 20);
// how long it takes for a rune to fade, must be negative!
const LIFETIME_FADE: i16 = -7;
// probability of .0 to .1 that generator spawns in a column per step
const GENERATOR_IN_COLUMN: (u16, u16) = (1, 50);
const RUNE_COLOR_BASE: (u8, u8, u8) = (0, 255, 255);
const RUNE_GENERATOR_COLOR: (u8, u8, u8) = (255, 0, 0);

struct Characters(&'static str);

impl Characters {
    fn create_random_rune(&self, color: (u8, u8, u8)) -> Rune {
        let mut rng = rand::thread_rng();
        let chars: Vec<char> = self.0.chars().collect();
        let idx = rng.gen_range(0..chars.len());
        let symbol = chars[idx];

        self.create_rune(symbol, color)
    }

    fn create_rune(&self, character: char, color: (u8, u8, u8)) -> Rune {
        let mut rng = rand::thread_rng();

        let lifetime = (rng.gen_range(LIFETIME.0..LIFETIME.1)) as i16;
        Rune {
            character,
            lifetime,
            color,
        }
    }
}
struct Grid(Vec<Vec<Rune>>);

impl Grid {
    fn new(characters: &Characters) -> Result<Self> {
        let (width, height) = terminal::size()?;
        let rune = characters.create_rune(' ', RUNE_COLOR_BASE);
        Ok(Grid(vec![vec![rune; width as usize]; height as usize]))
    }

    fn set_rune(&mut self, x: usize, y: usize, rune: Rune) -> Result<()> {
        *self
            .0
            .get_mut(y)
            .expect("out of bounds y Grid index")
            .get_mut(x)
            .expect("out of bounds x Grid index") = rune;
        Ok(())
    }

    fn get_rune(&mut self, x: usize, y: usize) -> Result<&mut Rune> {
        Ok(self
            .0
            .get_mut(y)
            .expect("out of bounds y Grid index")
            .get_mut(x)
            .expect("out of bounds y Grid index"))
    }
}

#[derive(Clone)]
struct Rune {
    character: char,
    lifetime: i16,
    color: (u8, u8, u8),
}

pub struct Waterfall<T: Write = Stdout> {
    grid: Grid,
    writer: T,
    generators: Vec<(usize, usize)>,
    characters: Characters,
    base_color: (u8, u8, u8),
}

impl Waterfall {
    pub fn new() -> Result<Self> {
        let symbols = Characters(SYMBOLS);
        let grid = Grid::new(&symbols)?;
        let mut stdout = io::stdout();

        stdout.queue(cursor::Hide)?;
        stdout.queue(Clear(ClearType::All))?;

        Ok(Waterfall {
            grid,
            generators: vec![],
            writer: stdout,
            characters: symbols,
            base_color: RUNE_COLOR_BASE,
        })
    }

    pub fn render(&mut self) -> Result<()> {
        for (y, row) in self.grid.0.iter().enumerate() {
            for (x, rune) in row.iter().enumerate() {
                let new_color = match rune.lifetime {
                    0.. => rune.color,
                    ..LIFETIME_FADE => (0, 0, 0),
                    v => (
                        rune.color.0 - (rune.color.0 / -LIFETIME_FADE as u8) * v.abs() as u8,
                        rune.color.1 - (rune.color.1 / -LIFETIME_FADE as u8) * v.abs() as u8,
                        rune.color.2 - (rune.color.2 / -LIFETIME_FADE as u8) * v.abs() as u8,
                    ),
                };

                self.writer
                    .queue(cursor::MoveTo(x as u16, y as u16))?
                    .queue(style::PrintStyledContent(
                        rune.character
                            .with(Color::Rgb {
                                r: new_color.0,
                                g: new_color.1,
                                b: new_color.2,
                            }) // .on(Color::Blue)
                            .attribute(Attribute::Encircled),
                    ))?
                    .queue(style::SetForegroundColor(Color::White))?;
            }
        }
        self.writer.flush()?;
        Ok(())
    }

    pub fn step(&mut self) -> Result<()> {
        for g in &self.generators {
            let rune = self.grid.get_rune(g.0, g.1)?;
            rune.color = self.base_color;
        }

        self.generators
            .retain(|g: &(usize, usize)| self.grid.0.len() > (g.1 + 1).into());

        let mut rng = rand::thread_rng();

        for g in self.generators.iter_mut() {
            g.1 += 1;
            let new_rune = self.characters.create_random_rune(self.base_color);
            self.grid.set_rune(g.0, g.1, new_rune)?;
        }
        for i in 0..self.grid.0[0].len() {
            if rng.gen_range(0..GENERATOR_IN_COLUMN.1) <= GENERATOR_IN_COLUMN.0 {
                self.generators.push((i, 0));
                let new_rune = self.characters.create_random_rune(self.base_color);
                self.grid.set_rune(i, 0, new_rune)?;
            }
        }

        for g in &self.generators {
            let rune = self.grid.get_rune(g.0, g.1)?;
            rune.color = RUNE_GENERATOR_COLOR;
        }

        for row in self.grid.0.iter_mut() {
            for rune in row {
                if LIFETIME.1 + LIFETIME_FADE.abs() as u16 > rune.lifetime.abs() as u16 {
                    rune.lifetime -= 1;
                }
            }
        }
        Ok(())
    }
}
