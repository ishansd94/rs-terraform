#![allow(unused)]

use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::{exit};
use crate::tf::executable;
use crate::tf::InputValues::Str;

mod tf;

fn init() {
    let terraform_bin: &'static [u8];
    let mut exec_post_fix =  "";

    match std::env::consts::OS {
        "windows" => {
            println!("Running in Windows");
            terraform_bin = include_bytes!("../bins/terraform.exe");
            exec_post_fix = ".exe"
        },
        "linux" => {
            println!("Running in Linux");
            terraform_bin = include_bytes!("../bins/terraform");
        },
        _ => {
            eprintln!("Unsupported OS");
            exit(1)
        }
    }
    let dir = tf::executable_path().clone();
    let executor_dir = Path::new(dir.as_str());

    if !executor_dir.exists() {
        match fs::create_dir(tf::executable_path()) {
            Ok(()) => {
                //TODO: add logger
                println!("Directory created at: {}", dir )
            }
            Err(e) => {
                eprintln!("Error creating directory: {}", e)
            }
        }
    }

    let mut terraform_exec = File::create(tf::executable()).unwrap();
    terraform_exec.write_all(terraform_bin).unwrap();
    drop(terraform_exec);
    // println!("Executable available at: {}", executable() )
}

fn main() {

    init();

    let mut tfbin_options = tf::ExecutorOptions::default();
    tfbin_options.static_workspace = true;
    tfbin_options.output = false;
    let mut tfbin = tf::Executor::new(tfbin_options);

    // tfbin.init("https://github.com/ishansd94/terraform-sample-module");
    let mut inputs = HashMap::new();
    inputs.insert(String::from("str"), tf::InputValues::Str(String::from("bar")));
    let _ = tfbin.set_inputs(inputs);
    // tfbin.apply();
    tfbin.output();
}


