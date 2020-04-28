#![allow(dead_code)]
#![allow(unused_variables)]
mod bc;
mod bc_protocol;
mod config;
mod cmdline;
mod gst;

use bc_protocol::BcCamera;
use config::{Config, CameraConfig};
use cmdline::Opt;
use crossbeam_utils::thread;
use err_derive::Error;
use gst::RtspServer;
use std::fs;
use std::io::Write;
use structopt::StructOpt;

#[derive(Debug, Error)]
pub enum Error {
    #[error(display="Configuration parsing error")]
    ConfigError(#[error(source)] toml::de::Error),
    #[error(display="Communication error")]
    ProtocolError(#[error(source)] bc_protocol::Error),
    #[error(display="I/O error")]
    IoError(#[error(source)] std::io::Error),
}

fn main() -> Result<(), Error> {
    let opt = Opt::from_args();
    let config: Config = toml::from_str(&fs::read_to_string(opt.config)?)?;

    let rtsp = &RtspServer::new();

    thread::scope(|s| {
        for camera in config.cameras {
            s.spawn(move |_| {
                // TODO handle these errors
                let mut output = rtsp.add_stream(&camera.name).unwrap(); // TODO
                camera_main(&camera, &mut output)
            });
        }

        rtsp.run(&config.bind_addr);
    }).unwrap();

    Ok(())
}

fn camera_main(camera_config: &CameraConfig, output: &mut dyn Write) -> Result<(), Error> {
    let mut camera = BcCamera::new_with_addr(camera_config.camera_addr)?;

    println!("{}: Connecting to camera at {}", camera_config.name, camera_config.camera_addr);

    camera.connect()?;
    camera.login(&camera_config.username, camera_config.password.as_deref())?;

    println!("{}: Connected to camera, starting video stream", camera_config.name);

    camera.start_video(output)?;

    Ok(())
}
