#![allow(unused)]
#![allow(clippy::needless_return)]

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{exit};
use dotenv::dotenv;
use log::{error, info};
use crate::tf::{executable, ExecutorOptions};
use crate::tf::InputValues::Str;

extern crate log;

mod tf;

fn init() {
    let terraform_bin: &'static [u8];
    let mut exec_post_fix =  "";

    match std::env::consts::OS {
        "windows" => {
            info!("pre-configure: running in Windows operating system.");
            terraform_bin = include_bytes!("../bins/terraform.exe");
            exec_post_fix = ".exe"
        },
        "linux" => {
            info!("pre-configure: running in Linux operating system.");
            terraform_bin = include_bytes!("../bins/terraform");
        },
        _ => {
            error!("pre-configure: unsupported operating system.");
            exit(1)
        }
    }
    let dir = tf::executable_path();
    let executor_dir = Path::new(dir.as_str());

    if !executor_dir.exists() {
        match fs::create_dir(tf::executable_path()) {
            Ok(()) => {
                info!("pre-configure: directory created at \"{}\"", dir);
            }
            Err(e) => {
                error!("pre-configure: failed to create directory at \"{}\", error: {}", dir, e)
            }
        }
    }

    let mut terraform_exec = File::create(tf::executable()).unwrap();
    terraform_exec.write_all(terraform_bin).unwrap();
    drop(terraform_exec);
}

fn main() {
    dotenv().ok();

    env_logger::init();

    init();

    let mut tfbin = tf::Executor::new(tf::ExecutorOptions{
        output: false,
        static_workspace: true,
        debug_mode: false,
    });

    tfbin.init("https://github.com/ishansd94/terraform-sample-module");
    let mut inputs = HashMap::new();
    inputs.insert(String::from("str"), tf::InputValues::Str(String::from("bar")));
    let _ = tfbin.set_inputs(inputs);
    tfbin.plan();
    tfbin.apply();
    println!("{:?}", tfbin.output());
    tfbin.destroy();
}


