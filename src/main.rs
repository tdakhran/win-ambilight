use std::{io::Write, time::Instant};

use windows_capture::{
    capture::GraphicsCaptureApiHandler,
    frame::Frame,
    graphics_capture_api::InternalCaptureControl,
    monitor::Monitor,
    settings::{ColorFormat, CursorCaptuerSettings, DrawBorderSettings, Settings},
};

use anyhow::Result;
use clap::{Parser, Subcommand};

include!(concat!(env!("OUT_DIR"), "/led_layout.rs"));
use root::{
    LedsBottomLeft, LedsBottomRight, LedsLeft, LedsRight, LedsTop, LedsTotal, SerialDataFooter,
    SerialDataHeader,
};

const SERIAL_PAYLOAD_SIZE: usize = 4 + LedsTotal * 3 + 4;

// for movies that have black bars on top and bottom
fn compute_vertical_blank_offset(width: usize, height: usize, img: &[u8]) -> usize {
    let mid = width / 4;
    let max_blank_stripe_height = height / 8;
    for row in 0..max_blank_stripe_height {
        let pixel_offset = 4 * (row * width + mid);
        let pixel = &img[pixel_offset..pixel_offset + 3];
        if pixel != [0, 0, 0] {
            return row;
        }
    }
    max_blank_stripe_height
}

fn compute_serial_payload(width: usize, height: usize, img: &[u8]) -> [u8; SERIAL_PAYLOAD_SIZE] {
    let mut payload = [0u8; SERIAL_PAYLOAD_SIZE];
    payload[0..4].copy_from_slice(&SerialDataHeader[0..4]);
    payload[4 + LedsTotal * 3..].copy_from_slice(&SerialDataFooter[0..4]);

    const DEPTH_PERCENT_FROM_SIZE: f32 = 0.02;
    let depth = (DEPTH_PERCENT_FROM_SIZE * (width as f32 + height as f32)) as usize;
    let mut offset: usize = 4;

    let blank_offset = compute_vertical_blank_offset(width, height, img);

    // compute average color for given rectangle and write it to payload
    let mut compute_avg_and_write =
        |row_start: usize, row_count: usize, col_start: usize, col_count: usize| {
            let mut r: u32 = 0;
            let mut g: u32 = 0;
            let mut b: u32 = 0;
            let mut total_pixels = 0;
            for row in row_start..row_start + row_count {
                for col in col_start..col_start + col_count {
                    b += img[4 * (row * width + col) + 0] as u32;
                    g += img[4 * (row * width + col) + 1] as u32;
                    r += img[4 * (row * width + col) + 2] as u32;
                    total_pixels += 1;
                }
            }

            if total_pixels > 0 {
                r /= total_pixels;
                g /= total_pixels;
                b /= total_pixels;
            }

            payload[offset] = r as u8;
            offset += 1;
            payload[offset] = g as u8;
            offset += 1;
            payload[offset] = b as u8;
            offset += 1;
        };

    assert!(LedsBottomLeft == LedsBottomRight);
    let horizontal_pixels_per_led: usize = width / (LedsTop + 2);
    assert!(LedsLeft == LedsRight);
    let vertical_pixels_per_led: usize = height / (LedsLeft + 2);

    // bottom right, left to right
    for id in 0..LedsBottomRight {
        compute_avg_and_write(
            height - depth - blank_offset,
            depth,
            width - horizontal_pixels_per_led * (LedsBottomRight + 1 - id),
            horizontal_pixels_per_led,
        );
    }

    // right, bottom to top
    for id in 0..LedsRight {
        compute_avg_and_write(
            height - vertical_pixels_per_led * (2 + id),
            vertical_pixels_per_led,
            width - depth,
            depth,
        );
    }

    // top, right to left
    for id in 0..LedsTop {
        compute_avg_and_write(
            blank_offset,
            depth,
            width - horizontal_pixels_per_led * (2 + id),
            horizontal_pixels_per_led,
        );
    }

    // left, top to bottom
    for id in 0..LedsLeft {
        compute_avg_and_write(
            vertical_pixels_per_led * (1 + id),
            vertical_pixels_per_led,
            0,
            depth,
        );
    }

    // bottom left, left to right
    for id in 0..LedsBottomLeft {
        compute_avg_and_write(
            height - depth - blank_offset,
            depth,
            horizontal_pixels_per_led * (1 + id),
            horizontal_pixels_per_led,
        );
    }

    payload
}

struct Capture {
    last_update_timestamp: Instant,
    port: Box<dyn serialport::SerialPort>,
    period: std::time::Duration,
}

impl GraphicsCaptureApiHandler for Capture {
    type Flags = (String, std::time::Duration);

    type Error = anyhow::Error;

    fn new(flags: Self::Flags) -> Result<Self, Self::Error> {
        let (com_port, period) = flags;

        let port = serialport::new(com_port, 230_400)
            .timeout(std::time::Duration::from_millis(10))
            .open()?;

        Ok(Self {
            last_update_timestamp: Instant::now(),
            port,
            period,
        })
    }

    fn on_frame_arrived(
        &mut self,
        frame: &mut Frame,
        _capture_control: InternalCaptureControl,
    ) -> Result<(), Self::Error> {
        let now = Instant::now();

        if now.duration_since(self.last_update_timestamp) < self.period {
            return Ok(());
        }

        self.last_update_timestamp = now;

        let payload = compute_serial_payload(
            frame.width() as usize,
            frame.height() as usize,
            &frame.buffer()?.as_raw_nopadding_buffer()?,
        );

        self.port.write(&payload)?;
        Ok(())
    }
}

#[derive(Parser)]
#[command(name = env!("CARGO_PKG_NAME"), version, about)]
struct WinAmbilight {
    #[command(subcommand)]
    pub command: WinAmbilightCommand,
}

#[derive(Subcommand)]
enum WinAmbilightCommand {
    /// List discovered monitors
    ListMonitors,
    /// List discovered serial ports
    ListSerialPorts,
    /// Start capture
    Run {
        #[clap(short)]
        port: String,
        #[clap(short)]
        monitor_index: usize,
    },
}

fn main() -> Result<()> {
    match WinAmbilight::parse().command {
        WinAmbilightCommand::ListMonitors => {
            for (index, monitor) in Monitor::enumerate()?.iter().enumerate() {
                println!(
                    "index: {index}, name: {} [{}x{}]",
                    monitor.device_string()?,
                    monitor.width()?,
                    monitor.height()?,
                );
            }
        }
        WinAmbilightCommand::ListSerialPorts => {
            for port in serialport::available_ports()? {
                println!("{}", port.port_name);
            }
        }
        WinAmbilightCommand::Run {
            port,
            monitor_index,
        } => {
            let monitor = Monitor::from_index(monitor_index + 1)?;

            let settings = Settings::new(
                monitor,
                CursorCaptuerSettings::WithoutCursor,
                DrawBorderSettings::WithoutBorder,
                ColorFormat::Rgba8,
                (port, std::time::Duration::from_millis(34)),
            )?;

            Capture::start(settings)?;
        }
    };

    Ok(())
}
