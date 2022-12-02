use super::data::ProgramFile;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "Compile", about = "Compile a miden program")]
pub struct CompileCmd {
    /// Path to .masm assembly file
    #[structopt(short = "a", long = "assembly", parse(from_os_str))]
    assembly_file: PathBuf,
}

impl CompileCmd {
    pub fn execute(&self) -> Result<(), String> {
        println!("============================================================");
        println!("Compile program");
        println!("============================================================");

        // load and compile program file
        let program = ProgramFile::read(&self.assembly_file)?;

        // report program hash to user
        let program_hash: [u8; 32] = program.hash().into();
        println!("program hash is {}", hex::encode(program_hash));

        Ok(())
    }
}
