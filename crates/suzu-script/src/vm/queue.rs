use std::collections::HashMap;

use super::Command;

#[derive(Debug, Clone, Default)]
pub struct CommandQueue {
    commands: Vec<Command>,
    pc: usize,
    call_stack: Vec<usize>,
    labels: HashMap<String, usize>,
}

impl CommandQueue {
    pub fn new(commands: Vec<Command>) -> Self {
        let labels = commands
            .iter()
            .enumerate()
            .filter_map(|(pc, command)| match command {
                Command::Label { name } => Some((name.clone(), pc)),
                _ => None,
            })
            .collect();

        Self {
            commands,
            labels,
            ..Self::default()
        }
    }

    pub fn next_command(&mut self) -> Option<&Command> {
        self.next_command_with_position()
            .map(|(_position, command)| command)
    }

    pub fn next_command_with_position(&mut self) -> Option<(usize, &Command)> {
        loop {
            let position = self.pc;
            let command = self.commands.get(position)?;
            self.pc += 1;
            if !matches!(command, Command::Label { .. }) {
                return Some((position, command));
            }
        }
    }

    pub fn len(&self) -> usize {
        self.commands.len()
    }

    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }

    pub fn jump_to(&mut self, label: &str) -> bool {
        if let Some(pc) = self.labels.get(label).copied() {
            self.pc = pc;
            true
        } else {
            false
        }
    }

    pub fn register_label(&mut self, label: impl Into<String>, pc: usize) {
        self.labels.insert(label.into(), pc);
    }

    pub fn push_return(&mut self) {
        self.call_stack.push(self.pc);
    }

    pub fn call(&mut self, label: &str) -> bool {
        if let Some(pc) = self.labels.get(label).copied() {
            self.call_stack.push(self.pc);
            self.pc = pc;
            true
        } else {
            false
        }
    }

    pub fn return_from_call(&mut self) -> bool {
        if let Some(pc) = self.call_stack.pop() {
            self.pc = pc;
            true
        } else {
            false
        }
    }

    pub fn call_stack(&self) -> &[usize] {
        &self.call_stack
    }

    pub fn set_call_stack(&mut self, call_stack: Vec<usize>) {
        self.call_stack = call_stack
            .into_iter()
            .map(|pc| pc.min(self.commands.len()))
            .collect();
    }

    pub fn position(&self) -> usize {
        self.pc
    }

    pub fn set_position(&mut self, pc: usize) {
        self.pc = pc.min(self.commands.len());
    }

    pub fn commands(&self) -> &[Command] {
        &self.commands
    }

    pub fn insert_next(&mut self, commands: Vec<Command>) {
        if commands.is_empty() {
            return;
        }

        let insert_at = self.pc;
        let inserted_len = commands.len();
        for pc in self.labels.values_mut() {
            if *pc >= insert_at {
                *pc += inserted_len;
            }
        }
        for (offset, command) in commands.iter().enumerate() {
            if let Command::Label { name } = command {
                self.labels.insert(name.clone(), insert_at + offset);
            }
        }
        self.commands.splice(insert_at..insert_at, commands);
    }
}
