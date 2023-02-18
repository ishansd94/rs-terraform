use std::borrow::Cow;
use std::collections::HashMap;
use std::env::args;
use std::fs;
use std::process::{Command, Output};
use serde_json::{json, to_string, Value};
use uuid::Uuid;

//Defaults related to the terraform executable
const EXECUTABLE_DIR_PATH: &str = ".";
const EXECUTABLE_DIR_NAME: &str = ".tf";
const EXECUTABLE_NAME: &str = "terraform";
const WORKSPACE_DIR: &str = "mod";

//Terraform main commands
const OPERATION_INIT: &str = "init";
const OPERATION_PLAN: &str = "plan";
const OPERATION_APPLY: &str = "apply";
const OPERATION_DESTROY: &str = "destroy";
const OPERATION_REFRESH: &str = "refresh";
const OPERATION_TAINT: &str = "taint";
const OPERATION_UNTAINT: &str = "untaint";
const OPERATION_SHOW: &str = "show";
const OPERATION_OUTPUT: &str = "output";
const OPERATION_STATE: &str = "state";

//Terraform optional arguments
const OPT_FROM_MODULE: &str = "-from-module";
const OPT_VAR: &str = "-var";

//Terraform options flags
const FLAG_AUTO_APPROVE: &str = "-auto-approve";
const FLAG_FORCE_COPY: &str = "-force-copy";
const FLAG_JSON: &str = "-json";

//Get path to the directory of terraform executable
pub fn executable_path() -> String {
    return format!("{}/{}", EXECUTABLE_DIR_PATH, EXECUTABLE_DIR_NAME);
}

//Get the full path to the terraform executable
pub fn executable() -> String {
    let mut ext = "";
    match std::env::consts::OS {
        "windows" => {
            ext = ".exe";
        },
        _ => {}
    }
    return format!("{}/{}{}",executable_path(),EXECUTABLE_NAME,ext);
}

//Terraform inputs
pub enum InputValues {
    Integer(i32),
    Float(f32),
    Str(String)
}

//Terraform executor
pub struct Executor{
    workspace: Workspace,
    inputs: HashMap<String, InputValues>,
    current_op: String,
    options: ExecutorOptions
}

//Options to configure the executor
#[derive(Default)]
pub struct ExecutorOptions {
    pub static_workspace: bool,
    pub debug_mode: bool,
    pub output: bool
}

//Dir configs for the terraform module
pub struct Workspace {
    dir: String,
    initialized: bool
}

//TODO: error handling
//Create the workspace, either uuid or static dir
fn create_workspace(static_workspace:bool) -> String{

    let uuid = Uuid::new_v4();
    let workspace: String = if !static_workspace { uuid.to_string() } else { WORKSPACE_DIR.to_string() };
    let dir = format!("{}/{}", executable_path().as_str(), workspace);
    match fs::create_dir(dir.clone()){
        Ok(()) => {
            println!("Using workspace: {}",dir.clone() );
        },
        Err(e) => {}
    }
    return dir.clone();
}

impl Executor {

    //Return new executor with base
    pub fn new(options: ExecutorOptions) -> Executor {
        let workspace = create_workspace(options.static_workspace);
        return Executor{
            workspace: Workspace{
                dir: workspace,
                initialized: false,
            },
            inputs: HashMap::new(),
            current_op: "".to_string(),
            options
        };
    }

    //Set terraform inputs
    pub fn set_inputs(&mut self, inputs: HashMap<String, InputValues>) -> &mut Executor{
        self.inputs = inputs;
        return self;
    }

    //Run terraform commands with args and flags
    fn run_command(&mut self, args: Vec<String>) -> Result<String, String> {

        let ex = executable();

        println!("Using executable at: {}", ex );
        println!("Using workspace at: {}", self.workspace.dir.clone() );
        println!("Using args: {:?}", args);

        let output = Command::new(ex)
                                .current_dir(self.workspace.dir.clone().as_str())
                                .args(args)
                                .output()
                                .expect("Failed to run command");
        // println!("{:?}", output);
        if !output.status.success() {
            let e = String::from_utf8_lossy(&output.stderr);
            eprintln!("Error: {}", e);
            return Err(e.to_string());
        }

        let o = String::from_utf8_lossy(&output.stdout);
        if self.options.output {
            println!("{}", o)
        }
        return Ok(o.to_string());
    }

    //Convert rust vars to a vector of terraform input strings '-var k=y' that can be passed to executable
    fn generate_inputs(&mut self) -> Vec<String> {

        let mut inputs = vec![];

        match self.current_op.as_str() {
            OPERATION_APPLY|OPERATION_PLAN|OPERATION_DESTROY|OPERATION_REFRESH => {
                inputs = self.inputs
                    .iter()
                    .map(|(k,v)| {
                        let input = match v {
                            InputValues::Float(i) => i.to_string(),
                            InputValues::Integer(i) => i.to_string(),
                            InputValues::Str(i) => i.clone(),
                        };
                        return format!("-var={}={}", k, input);
                    })
                    .collect()
            }
            OPERATION_OUTPUT => {}
            _ => {}
        }

        return inputs
    }

    //Generate the options based on the main command
    fn generate_options(&mut self) -> Vec<String> {

        return vec![];
    }

    //Generate flags based on the main command
    fn generate_flags(&mut self) -> Vec<String> {

        let mut flags = vec![];
        match self.current_op.as_str() {
            OPERATION_APPLY|OPERATION_PLAN|OPERATION_DESTROY|OPERATION_REFRESH => {
                flags.push(FLAG_AUTO_APPROVE.to_string())
            }
            OPERATION_OUTPUT => {
                flags.push(FLAG_JSON.to_string())
            }
            _ => {}
        }
        return flags;
    }

    fn build(&mut self, command: String) -> Vec<String> {
        let mut args = vec![command];
        let inputs = self.generate_inputs();
        let flags = self.generate_flags();
        let opt = self.generate_options();
        args.extend(inputs);
        args.extend(flags);
        args.extend(opt);
        return args
    }

    //terraform init
    pub fn init(&mut self, source: &str) {
        self.current_op = OPERATION_INIT.to_string();
        if self.workspace.initialized {
            return;
        }
        self.run_command(vec![OPERATION_INIT.to_string(), format!("{}=git::{}", OPT_FROM_MODULE, source)]);
        self.workspace.initialized = true;
    }

    //terraform apply
    pub fn apply(&mut self) {
        self.current_op = OPERATION_APPLY.to_string();
        let args = self.build(OPERATION_APPLY.to_string());
        self.run_command(args);
    }

    pub fn output(&mut self) -> Result<Value, String> {
        self.current_op = OPERATION_OUTPUT.to_string();
        let args = self.build(OPERATION_OUTPUT.to_string());
        let output  = self.run_command(args)?;
        let value: Value = serde_json::from_str(output.clone().as_str()).map_err(|e| e.to_string())?;
        return Ok(value);
    }
}
