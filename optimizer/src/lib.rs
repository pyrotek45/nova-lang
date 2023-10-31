use common::code::Asm;

pub struct Optimizer {
    pub optimizations: usize,
}

pub fn new() -> Optimizer {
    Optimizer { optimizations: 0 }
}

impl Optimizer {
    pub fn optimize(&mut self, _input: Vec<Asm>) -> Vec<Asm> {
        // first pass
        //self.tail_call(input)
        todo!()
    }

    // under construction
    // fn tail_call(&mut self, input: Vec<Asm>) -> Vec<Asm> {
    //     let mut output = vec![];
    //     let mut function_stack = vec![];
    //     for instruction in input {
    //         match instruction.clone() {
    //             Asm::FUNCTION(index) => {
    //                 function_stack.push((index, name));
    //                 output.push(instruction);
    //             }
    //             Asm::LABEL(label) => {
    //                 if let Some((index, _)) = function_stack.last() {
    //                     if label == *index {
    //                         function_stack.pop();
    //                     }
    //                 }
    //                 output.push(instruction);
    //             }
    //             Asm::RET(_) => {
    //                 if let Some((_, name)) = function_stack.last() {
    //                     if let Some(Asm::DCALL(dindex)) = output.clone().last() {
    //                         if &name == &dname {
    //                             output.pop();
    //                             output.push(Asm::TCALL(*dindex));
    //                             self.optimizations += 1;
    //                         }
    //                     }
    //                 }
    //                 output.push(instruction);
    //             }
    //             _ => {
    //                 output.push(instruction);
    //             }
    //         }
    //     }
    //  output
    //}
}
