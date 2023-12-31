use std::error::Error;
use std::fs::File;
use std::process;
use std::io::{self, Read};
use csv::ReaderBuilder;
use std::env;

#[derive(Debug)]
struct Instruction {
    opcode: u32,
    name: String,
    size: usize,
}

fn parse_instructions(file_path: &str) -> Result<Vec<Instruction>, Box<dyn Error>> {
    let file = File::open(file_path)?;
    let mut reader = ReaderBuilder::new().from_reader(file);
    let mut instructions = Vec::new();

    for result in reader.records() {
        let record = result?;
        let instruction = Instruction {
            opcode: record[0].parse()?,
            name: record[1].to_string(),
            size: record[2].parse()?,
        };
        
        instructions.push(instruction);
    }

    Ok(instructions)
}

fn read_binary(file_path: &str) -> io::Result<Vec<u8>> {
    let mut file = File::open(file_path)?;
    let mut buffer = Vec::new();

    file.read_to_end(&mut buffer)?;

    Ok(buffer)
}

fn disassemble_code(instructions: &[Instruction], code: &[u8]) {
    let mut offset: usize = 0;

    while offset < code.len() {
        let opcode = code[offset] as u32;

        if let Some(instruction) = instructions.iter().find(|&instr| instr.opcode == opcode) {
            let mut op_string = String::new();
            
            for i in 0..=(instruction.size - 1) {
                op_string.push_str(&format!(" {:02X}", code[offset + i]));
            }

            println!(
               "{:04X} - {} ({})", 
                offset, op_string, instruction.name
            );

            offset += instruction.size as usize;
        } else {
            eprintln!("Error: Unknow opcode {:02X} at position {:04X}", opcode, offset);
            break;
        }
    }
    
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 {
        eprintln!("Usage: {} <csv_file_path> <rom_file_path>", args[0]);
        process::exit(1);
    }

    let csv_file_path = &args[1];
    let rom_file_path = &args[2];

    if let Ok(instructions) = parse_instructions(csv_file_path) {
        if let Ok(rom_buffer) = read_binary(rom_file_path) {
            disassemble_code(&instructions, &rom_buffer);
        } else {
            eprintln!("Error reading ROM file");
        }
    } else {
        eprintln!("Error reading CSV file");
    }
}
