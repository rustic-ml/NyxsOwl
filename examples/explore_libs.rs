// This program explores the module structure of rustalib and oxidiviner libraries
fn main() {
    println!("Exploring libraries");
    
    // Use the libraries to see what modules and functions are available
    // This will be visible in compiler errors
    
    // Rustalib
    let _ = rustalib::version();
    
    // Oxidiviner
    let _ = oxidiviner::version();
    
    println!("Done");
} 