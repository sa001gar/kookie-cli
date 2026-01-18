//! Input utilities for interactive prompts

use colored::*;
use rpassword::read_password;
use std::io::{self, Write};

/// Prompts for a password (hidden input)
pub fn prompt_password(prompt: &str) -> io::Result<String> {
    print!("{} ", prompt.cyan());
    io::stdout().flush()?;
    read_password()
}

/// Prompts for text input
pub fn prompt_text(prompt: &str) -> io::Result<String> {
    print!("{} ", prompt.cyan());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

/// Prompts for optional text input
pub fn prompt_optional(prompt: &str) -> io::Result<Option<String>> {
    let input = prompt_text(prompt)?;
    if input.is_empty() {
        Ok(None)
    } else {
        Ok(Some(input))
    }
}

/// Prompts for confirmation (y/n)
pub fn prompt_confirm(prompt: &str, default: bool) -> io::Result<bool> {
    let suffix = if default { "[Y/n]" } else { "[y/N]" };
    print!("{} {} ", prompt.cyan(), suffix.dimmed());
    io::stdout().flush()?;
    
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let input = input.trim().to_lowercase();
    
    if input.is_empty() {
        Ok(default)
    } else {
        Ok(input == "y" || input == "yes")
    }
}

/// Prompts for a password with confirmation
pub fn prompt_new_password(prompt: &str) -> io::Result<String> {
    loop {
        let password = prompt_password(prompt)?;
        
        if password.len() < 8 {
            println!("{}", "Password must be at least 8 characters.".red());
            continue;
        }
        
        let confirm = prompt_password("Confirm password:")?;
        
        if password != confirm {
            println!("{}", "Passwords do not match. Try again.".red());
            continue;
        }
        
        return Ok(password);
    }
}

/// Prompts for a number
#[allow(dead_code)]
pub fn prompt_number(prompt: &str, default: Option<u32>) -> io::Result<u32> {
    loop {
        let default_str = default.map(|d| format!(" [{}]", d)).unwrap_or_default();
        print!("{}{}: ", prompt.cyan(), default_str.dimmed());
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        
        if input.is_empty() {
            if let Some(d) = default {
                return Ok(d);
            }
            println!("{}", "Please enter a number.".red());
            continue;
        }
        
        match input.parse::<u32>() {
            Ok(n) => return Ok(n),
            Err(_) => {
                println!("{}", "Invalid number. Try again.".red());
                continue;
            }
        }
    }
}
