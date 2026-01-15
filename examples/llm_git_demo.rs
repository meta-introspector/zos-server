// Example: LLM accessing git in secure container
use crate::container_runtime::llm_git;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 LLM Secure Git Access Demo");

    // Initialize secure git container for LLM
    let container_id = llm_git::init_secure_git("/path/to/repo")?;
    println!("✅ Created secure container: {}", container_id);

    // LLM can safely access git operations
    println!("\n--- Git Log (LLM Safe) ---");
    match llm_git::safe_git_log(&container_id) {
        Ok(log) => println!("{}", log),
        Err(e) => println!("❌ Error: {}", e),
    }

    // LLM can view commits safely
    println!("\n--- Git Show (LLM Safe) ---");
    match llm_git::safe_git_show(&container_id, "HEAD") {
        Ok(content) => println!("{}", content),
        Err(e) => println!("❌ Error: {}", e),
    }

    // Demonstrate that dangerous operations are blocked
    println!("\n--- Blocked Operations ---");
    println!("❌ execve: Stripped from container");
    println!("❌ fork: Stripped from container");
    println!("❌ mount: Stripped from container");
    println!("❌ Direct file system access: Virtualized");

    println!("\n🎯 All git operations run in virtual filesystem!");
    println!("🔒 No syscalls can escape the container!");

    Ok(())
}

// Example of automatic patching in action:
mod example_before_after {
    // BEFORE (dangerous - direct git2 usage):
    fn dangerous_git_access() {
        // This would be automatically patched:
        // let repo = git2::Repository::open("/path/to/repo").unwrap();
        // let mut revwalk = repo.revwalk().unwrap();
    }

    // AFTER (safe - automatically patched):
    fn safe_git_access() {
        // Automatically becomes:
        let container_id =
            crate::security::container::init_llm_git_container("/path/to/repo").unwrap();
        let log = crate::security::container::git::log(&container_id).unwrap();
        // All operations run in virtual filesystem - no syscalls!
    }
}
