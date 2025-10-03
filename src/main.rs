use clap::{Arg, Command};
use std::fs::OpenOptions;
use std::io::Write;
use std::thread::sleep;
use std::time::{Duration, Instant};

const MNT_KEYBOARD4_HIDRAW_DEVICE: &str = "/dev/hidraw0";

const RAINBOW_RGB: [(u8, u8, u8); 12] = [
    (255, 0, 0),    // Red
    (255, 63, 0),
    (255, 127, 0),  // Orange
    (255, 191, 0),
    (255, 255, 0),  // Yellow
    (127, 255, 0),
    (0, 255, 0),    // Green
    (0, 127, 127),
    (0, 0, 255),    // Blue
    (37, 0, 192),
    (111, 0, 170),
    (148, 0, 211),  // Indigo
];

const NB_LED_ROWS: usize = 6;
const NB_LED_COLS: usize = 14;

fn parse_intensity(value: &str) -> Result<f32, String> {
    let v: f32 = value.parse().map_err(|_| format!("{}: invalid intensity", value))?;
    if v < 0.0 {
        return Err(format!("{}: intensity too low (min. 0%)", v));
    }
    if v > 100.0 {
        return Err(format!("{}: intensity too high (max. 100%)", v));
    }
    Ok(v)
}

fn parse_delay(value: &str) -> Result<f32, String> {
    let v: f32 = value.parse().map_err(|_| format!("{}: invalid delay", value))?;
    if v < 1.0 {
        return Err(format!("{}: delay too low (min. 1s)", v));
    }
    Ok(v)
}

fn main() {
    let matches = Command::new("mnt_reform_keyboard_backlight_rainbow")
        .arg(
            Arg::new("intensity")
                .short('i')
                .long("intensity")
                .value_parser(parse_intensity)
                .default_value("50")
                .help("Backlight intensity (default: 50%)"),
        )
        .arg(
            Arg::new("refresh_every_sec")
                .short('r')
                .long("refresh-every-sec")
                .value_parser(parse_delay)
                .help("Number of seconds between refreshes (default: no refresh)"),
        )
        .get_matches();

    let intensity: f32 = *matches.get_one::<f32>("intensity").unwrap();
    let refresh_every_sec: Option<f32> = matches.get_one::<f32>("refresh_every_sec").copied();

    let nb_colors = RAINBOW_RGB.len();

    loop {
        for seq in 0..nb_colors {
            let start_time = Instant::now();

            // Build one row of BGR values, flattened
            let mut row_bgr: Vec<u8> = Vec::new();
            for i in 0..NB_LED_COLS {
                let rgb = RAINBOW_RGB[(seq + i) % nb_colors];
                // reverse to BGR
                let bgr = [rgb.2, rgb.1, rgb.0];
                for v in bgr.iter() {
                    row_bgr.push((*v as f32 * intensity / 100.0).round() as u8);
                }
            }

            let mut row = 0;
            while row < NB_LED_ROWS {
                let result = (|| -> std::io::Result<()> {
                    let mut file = OpenOptions::new()
                        .write(true)
                        .open(MNT_KEYBOARD4_HIDRAW_DEVICE)?;
                    let mut data = vec![b'x', b'X', b'R', b'G', b'B'];
                    data.push(row as u8);
                    data.extend_from_slice(&row_bgr);
                    file.write_all(&data)?;
                    Ok(())
                })();

                match result {
                    Ok(_) => row += 1,
                    Err(_) => {
                        row = 0; // retry entire refresh
                    }
                }

                sleep(Duration::from_millis(150));
            }

            if refresh_every_sec.is_none() {
                return;
            }

            let elapsed = start_time.elapsed().as_secs_f32();
            let wait_time = refresh_every_sec.unwrap() - elapsed;
            if wait_time > 0.0 {
                sleep(Duration::from_secs_f32(wait_time));
            }
        }
    }
}

