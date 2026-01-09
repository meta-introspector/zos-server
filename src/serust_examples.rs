// SErust Examples - Demonstrating Friendly Declarative Security
use crate::serust_macros::*;
use crate::serust_lints::serust_runtime;

/// Example 1: Simple L0 Public Function
#[serust_domain(level = 0, orbits = "trivial", capabilities = "read,compute")]
#[orbit(trivial)]
#[track_provenance]
pub fn public_calculator(a: i32, b: i32, op: &str) -> Result<i32, String> {
    match op {
        "add" => Ok(a + b),
        "sub" => Ok(a - b),
        "mul" => Ok(a * b),
        "div" if b != 0 => Ok(a / b),
        "div" => Err("Division by zero".to_string()),
        _ => Err("Unknown operation".to_string()),
    }
}

/// Example 2: L1 System Function with Syscall Restrictions
#[serust_domain(level = 1, orbits = "trivial,cyclic", capabilities = "read,compute,file")]
#[orbit(cyclic)]
#[requires(file)]
#[allow_syscalls("read", "write", "open", "close")]
#[track_provenance]
pub fn safe_file_processor(filename: &str) -> Result<String, String> {
    // This would normally use std::fs, but we're demonstrating the concept
    println!("Processing file: {}", filename);
    Ok(format!("Processed: {}", filename))
}

/// Example 3: L2 Data Function with Complex Orbit
#[serust_domain(level = 2, orbits = "trivial,cyclic,symmetric", capabilities = "read,compute,data,transform")]
#[orbit(symmetric)]
#[requires(data, transform)]
#[track_provenance]
pub fn data_transformer<T: Clone + Ord>(mut data: Vec<T>) -> Vec<T> {
    data.sort(); // O(n log n) - symmetric group operation
    data
}

/// Example 4: L3 Admin Function with High Privileges
#[serust_domain(level = 3, orbits = "trivial,cyclic,symmetric,alternating", capabilities = "read,compute,admin,system")]
#[orbit(alternating)]
#[requires(admin, system)]
#[allow_syscalls("read", "write", "execve", "fork")]
#[track_provenance]
pub fn admin_system_operation(command: &str) -> Result<String, String> {
    if !command.starts_with("safe_") {
        return Err("Only safe commands allowed".to_string());
    }

    println!("Executing admin command: {}", command);
    Ok(format!("Executed: {}", command))
}

/// Example 5: L4 Kernel Function (Unrestricted)
#[serust_domain(level = 4, orbits = "trivial,cyclic,symmetric,alternating,sporadic,monster", capabilities = "all")]
#[orbit(monster)]
#[requires(kernel)]
#[track_provenance]
pub fn kernel_operation(operation: &str, data: &[u8]) -> Result<Vec<u8>, String> {
    println!("Kernel operation: {} with {} bytes", operation, data.len());
    Ok(data.to_vec())
}

/// Example 6: Function that would trigger Clippy warnings
pub fn unsafe_function() {
    // This would trigger MISSING_ORBIT_CLASSIFICATION
    // This would trigger MISSING_DOMAIN_RESTRICTION

    // This would trigger POTENTIAL_SYSCALL_USAGE
    std::process::Command::new("rm").arg("-rf").arg("/");

    // This would trigger MISSING_PROVENANCE_TRACKING
    std::fs::read_to_string("important_file.txt");
}

/// Example 7: Properly annotated version
#[serust_domain(level = 1, orbits = "trivial", capabilities = "read,file")]
#[orbit(trivial)]
#[requires(file)]
#[allow_syscalls("read", "open", "close")]
#[track_provenance]
pub fn safe_function() -> Result<String, String> {
    // Virtual file system operation instead of direct syscall
    Ok("File content".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Generated compliance tests
    test_orbit_compliance!(function = "public_calculator", orbit = "trivial");
    test_orbit_compliance!(function = "safe_file_processor", orbit = "cyclic");
    test_orbit_compliance!(function = "data_transformer", orbit = "symmetric");
    test_orbit_compliance!(function = "admin_system_operation", orbit = "alternating");
    test_orbit_compliance!(function = "kernel_operation", orbit = "monster");

    // Security domain tests
    security_test!(name = "public_calc_in_l0", domain = "l0_public", should_allow = true);
    security_test!(name = "admin_op_in_l0", domain = "l0_public", should_allow = false);
    security_test!(name = "kernel_op_in_l4", domain = "l4_kernel", should_allow = true);

    #[test]
    fn test_public_calculator() {
        let result = public_calculator(2, 3, "add");
        assert_eq!(result, Ok(5));

        let result = public_calculator(10, 0, "div");
        assert!(result.is_err());
    }

    #[test]
    fn test_data_transformer() {
        let data = vec![3, 1, 4, 1, 5, 9, 2, 6];
        let sorted = data_transformer(data);
        assert_eq!(sorted, vec![1, 1, 2, 3, 4, 5, 6, 9]);
    }

    #[test]
    fn test_admin_system_operation() {
        let result = admin_system_operation("safe_backup");
        assert!(result.is_ok());

        let result = admin_system_operation("rm -rf /");
        assert!(result.is_err());
    }
}

/// Example usage in main
pub fn demonstrate_serust() {
    println!("🚀 SErust Security Demonstration");

    // L0 Public operations
    println!("\n📊 L0 Public Domain:");
    match public_calculator(10, 5, "add") {
        Ok(result) => println!("  Calculator: 10 + 5 = {}", result),
        Err(e) => println!("  Error: {}", e),
    }

    // L1 System operations
    println!("\n🔧 L1 System Domain:");
    match safe_file_processor("test.txt") {
        Ok(result) => println!("  File processor: {}", result),
        Err(e) => println!("  Error: {}", e),
    }

    // L2 Data operations
    println!("\n📈 L2 Data Domain:");
    let data = vec![5, 2, 8, 1, 9];
    let sorted = data_transformer(data);
    println!("  Data transformer: {:?}", sorted);

    // L3 Admin operations
    println!("\n🔐 L3 Admin Domain:");
    match admin_system_operation("safe_maintenance") {
        Ok(result) => println!("  Admin operation: {}", result),
        Err(e) => println!("  Error: {}", e),
    }

    // L4 Kernel operations
    println!("\n⚡ L4 Kernel Domain:");
    match kernel_operation("memory_map", &[1, 2, 3, 4]) {
        Ok(result) => println!("  Kernel operation: {} bytes processed", result.len()),
        Err(e) => println!("  Error: {}", e),
    }

    println!("\n✅ All security domains demonstrated successfully!");
}
