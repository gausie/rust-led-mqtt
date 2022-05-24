use embedded_graphics::{
    geometry::{Point, Size},
    mono_font::{ascii::FONT_5X7, MonoTextStyle},
    pixelcolor::Rgb888,
    prelude::*,
    primitives::{PrimitiveStyleBuilder, Rectangle},
    text::Text,
};
use rpi_led_matrix::{LedCanvas, LedColor, LedMatrix, LedMatrixOptions};

use std::sync::mpsc;

fn create_matrix() -> Result<LedMatrix, &'static str> {
    let mut options = LedMatrixOptions::new();
    options.set_hardware_mapping("adafruit-hat");
    options.set_rows(16);
    options.set_cols(32);
    options.set_multiplexing(8);
    options.set_refresh_rate(false);

    LedMatrix::new(Some(options), None)
}

enum Scene<'a> {
    Blank { matrix: &'a LedMatrix },
    ColourCycle { matrix: &'a LedMatrix, step: i32 },
    OnAir { matrix: &'a LedMatrix },
}

impl Iterator for Scene<'_> {
    type Item = LedCanvas;

    fn next(&mut self) -> Option<LedCanvas> {
        match self {
            Scene::Blank { matrix } => {
                let mut canvas = matrix.offscreen_canvas();
                canvas.fill(&LedColor {
                    red: 0,
                    green: 0,
                    blue: 0,
                });
                canvas = matrix.swap(canvas);
                Some(canvas)
            }
            Scene::ColourCycle { matrix, step } => {
                let mut canvas = matrix.offscreen_canvas();

                let red = ((*step & 0b111111110000000000000000) >> 16) as u8;
                let green = ((*step & 0b1111111100000000) >> 8) as u8;
                let blue = ((*step & 0b11111111) >> 0) as u8;

                canvas.fill(&LedColor { red, green, blue });

                *step += 1;

                canvas = matrix.swap(canvas);

                Some(canvas)
            }
            Scene::OnAir { matrix } => {
                let mut canvas = matrix.offscreen_canvas();

                let line_style = PrimitiveStyleBuilder::new()
                    .stroke_color(Rgb888::RED)
                    .stroke_width(1)
                    .build();

                Rectangle::new(Point::zero(), Size::new(32, 16))
                    .into_styled(line_style)
                    .draw(&mut canvas)
                    .ok()?;

                let text_style = MonoTextStyle::new(&FONT_5X7, Rgb888::RED);

                // We want monospaced...
                Text::new("ON", Point::new(3, 10), text_style)
                    .draw(&mut canvas)
                    .ok()?;
                // ... except we want a smaller space :)
                Text::new("AIR", Point::new(15, 10), text_style)
                    .draw(&mut canvas)
                    .ok()?;

                canvas = matrix.swap(canvas);

                Some(canvas)
            }
        }
    }
}

pub fn run(rx: mpsc::Receiver<String>) {
    let matrix = create_matrix().unwrap();

    let mut previous_scene_id = String::from("blank");

    let mut scene = Scene::Blank { matrix: &matrix };

    loop {
        match rx.try_recv() {
            Ok(new_scene_id) => {
                if new_scene_id != previous_scene_id {
                    match new_scene_id.as_str() {
                        "colourcycle" => {
                            scene = Scene::ColourCycle {
                                matrix: &matrix,
                                step: 0,
                            };
                        }
                        "onair" => {
                            scene = Scene::OnAir { matrix: &matrix };
                        }
                        _ => {
                            scene = Scene::Blank { matrix: &matrix };
                        }
                    }
                    previous_scene_id = new_scene_id;
                }
            }
            _ => {}
        }

        scene.next().unwrap();
    }
}
