use std::error::Error;
use std::fs::File;
use std::process;
use std::io::{self, Read};
use csv::ReaderBuilder;
use std::env;

#[derive(Debug)]
struct Instruction {
    opcode: u8,
    name: String,
    size: usize,
}

pub struct ConditionCodes {
    z: bool,
    s: bool,
    p: bool,
    cy: bool,
    ac: bool,
    pad: u8,
}

fn default_codes() -> ConditionCodes {
    return ConditionCodes {
        z: true,
        s: true,
        p: true,
        cy: true,
        ac: true,
        pad: 3,
    };
}

pub struct State8080 {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    sp: u16,
    pc: usize,
    memory: Vec<u8>,
    cc: ConditionCodes,
    int_enable: bool,
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
        let opcode = code[offset] as u8;

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

fn unimplemented_instruction(state: &State8080) {
    println!("Unrecognized instruction called.");
    process::exit(1);
}

fn parity(value: u16) -> bool {
    let set_bits_count = value.count_ones();

    set_bits_count % 2 == 0
}

fn add_operation(state: &mut State8080, register_value: u8) {
    let answer = state.a as u16 + register_value as u16;
    state.cc.z = (answer & 0xff) == 0;
    state.cc.s = (answer & 0x80) != 0;
    state.cc.cy = answer > 0xff;
    state.cc.p = parity(answer & 0xff);
    state.a = (answer & 0xff) as u8;
}

fn run_emulation(state: &mut State8080, instructions: &[Instruction]) {
    let opcode = state.memory[state.pc];

    match opcode {
        0 => {},
        1 => {
            state.c = state.memory[state.pc + 1];
            state.b = state.memory[state.pc + 2];
        },
        128 => add_operation(state, state.b),
        129 => add_operation(state, state.c),
        130 => add_operation(state, state.d),
        131 => add_operation(state, state.e),
        132 => add_operation(state, state.h),
        133 => add_operation(state, state.l),
        //134 => add_operation(state, state.m),
        135 => add_operation(state, state.a),
        198 => add_operation(state, state.memory[state.pc + 1]),
        _ => unimplemented_instruction(state),
    }

    if let Some(instruction) = instructions.iter().find(|&instr| instr.opcode == opcode) {
        state.pc += instruction.size;
    } else {
        eprintln!("Error: Unknow opcode {:02X} at position {:04X}", opcode, state.pc);
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
