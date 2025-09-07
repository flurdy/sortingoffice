//! Rust-based container management for test cleanup
//!
//! This module provides Rust alternatives to docker CLI calls for better
//! cross-platform compatibility and performance by leveraging the testcontainers crate.

use anyhow::{Context, Result};
use std::process::Command;

/// Container information for cleanup operations
#[derive(Debug, Clone)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
}

/// Network information for cleanup operations
#[derive(Debug, Clone)]
pub struct NetworkInfo {
    pub id: String,
    pub name: String,
    pub driver: String,
}

/// Volume information for cleanup operations
#[derive(Debug, Clone)]
pub struct VolumeInfo {
    pub name: String,
    pub driver: String,
}

/// Rust-based container manager that uses testcontainers and shell fallbacks
pub struct ContainerManager;

impl Default for ContainerManager {
    fn default() -> Self {
        Self
    }
}

impl ContainerManager {
    /// Create a new container manager instance
    pub fn new() -> Self {
        Self
    }

    /// List all containers using docker CLI with structured output
    pub async fn list_containers(&self) -> Result<Vec<ContainerInfo>> {
        let output = Command::new("docker")
            .args([
                "ps",
                "-a",
                "--format",
                "{{.ID}}\t{{.Names}}\t{{.Image}}\t{{.Status}}",
            ])
            .output()
            .context("Failed to execute docker ps command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Docker ps failed: {}", stderr));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let container_infos: Vec<ContainerInfo> = output_str
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 4 {
                    Some(ContainerInfo {
                        id: parts[0].to_string(),
                        name: parts[1].trim_start_matches('/').to_string(),
                        image: parts[2].to_string(),
                        status: parts[3].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(container_infos)
    }

    /// Find containers by image pattern
    pub async fn find_containers_by_image(
        &self,
        image_pattern: &str,
    ) -> Result<Vec<ContainerInfo>> {
        let containers = self.list_containers().await?;

        Ok(containers
            .into_iter()
            .filter(|container| container.image.contains(image_pattern))
            .collect())
    }

    /// Find containers by name pattern
    pub async fn find_containers_by_name(&self, name_pattern: &str) -> Result<Vec<ContainerInfo>> {
        let containers = self.list_containers().await?;

        Ok(containers
            .into_iter()
            .filter(|container| container.name.contains(name_pattern))
            .collect())
    }

    /// Remove a container by ID using docker CLI
    pub async fn remove_container(&self, container_id: &str, force: bool) -> Result<()> {
        let mut args = vec!["rm"];
        if force {
            args.push("-f");
        }
        args.push(container_id);

        let output = Command::new("docker")
            .args(&args)
            .output()
            .context(format!(
                "Failed to execute docker rm command for {container_id}"
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("No such container") {
                return Err(anyhow::anyhow!(
                    "Failed to remove container {}: {}",
                    container_id,
                    stderr
                ));
            }
        }

        Ok(())
    }

    /// Remove multiple containers
    pub async fn remove_containers(&self, container_ids: &[String], force: bool) -> Result<()> {
        let mut results = Vec::new();

        for container_id in container_ids {
            match self.remove_container(container_id, force).await {
                Ok(()) => results.push(Ok(())),
                Err(e) => {
                    eprintln!("[CLEANUP] Warning: Failed to remove container {container_id}: {e}");
                    results.push(Err(e));
                }
            }
        }

        // Return success if at least one container was removed
        if results.iter().any(|r| r.is_ok()) {
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to remove any containers"))
        }
    }

    /// List all networks using docker CLI
    pub async fn list_networks(&self) -> Result<Vec<NetworkInfo>> {
        let output = Command::new("docker")
            .args([
                "network",
                "ls",
                "--format",
                "{{.ID}}\t{{.Name}}\t{{.Driver}}",
            ])
            .output()
            .context("Failed to execute docker network ls command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Docker network ls failed: {}", stderr));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let network_infos: Vec<NetworkInfo> = output_str
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 3 {
                    Some(NetworkInfo {
                        id: parts[0].to_string(),
                        name: parts[1].to_string(),
                        driver: parts[2].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(network_infos)
    }

    /// Remove a network by name using docker CLI
    pub async fn remove_network(&self, network_name: &str) -> Result<()> {
        let output = Command::new("docker")
            .args(["network", "rm", network_name])
            .output()
            .context(format!(
                "Failed to execute docker network rm command for {network_name}"
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("No such network") {
                return Err(anyhow::anyhow!(
                    "Failed to remove network {}: {}",
                    network_name,
                    stderr
                ));
            }
        }

        Ok(())
    }

    /// List all volumes using docker CLI
    pub async fn list_volumes(&self) -> Result<Vec<VolumeInfo>> {
        let output = Command::new("docker")
            .args(["volume", "ls", "--format", "{{.Name}}\t{{.Driver}}"])
            .output()
            .context("Failed to execute docker volume ls command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("Docker volume ls failed: {}", stderr));
        }

        let output_str = String::from_utf8_lossy(&output.stdout);
        let volume_infos: Vec<VolumeInfo> = output_str
            .lines()
            .filter_map(|line| {
                let parts: Vec<&str> = line.split('\t').collect();
                if parts.len() >= 2 {
                    Some(VolumeInfo {
                        name: parts[0].to_string(),
                        driver: parts[1].to_string(),
                    })
                } else {
                    None
                }
            })
            .collect();

        Ok(volume_infos)
    }

    /// Remove a volume by name using docker CLI
    pub async fn remove_volume(&self, volume_name: &str) -> Result<()> {
        let output = Command::new("docker")
            .args(["volume", "rm", volume_name])
            .output()
            .context(format!(
                "Failed to execute docker volume rm command for {volume_name}"
            ))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if !stderr.contains("No such volume") {
                return Err(anyhow::anyhow!(
                    "Failed to remove volume {}: {}",
                    volume_name,
                    stderr
                ));
            }
        }

        Ok(())
    }

    /// Clean up orphaned test containers
    pub async fn cleanup_orphaned_test_containers(&self) -> Result<()> {
        println!("[RUST CLEANUP] Cleaning up orphaned test containers...");

        // Find MySQL test containers
        let mysql_containers = self.find_containers_by_image("mysql").await?;
        let test_mysql_containers: Vec<String> = mysql_containers
            .into_iter()
            .filter(|container| container.name.contains("test") || container.name.contains("mysql"))
            .map(|container| container.id)
            .collect();

        if !test_mysql_containers.is_empty() {
            println!(
                "[RUST CLEANUP] Found {} orphaned MySQL test containers",
                test_mysql_containers.len()
            );
            self.remove_containers(&test_mysql_containers, true).await?;
        }

        // Find Selenium test containers
        let selenium_containers = self.find_containers_by_image("selenium").await?;
        let test_selenium_containers: Vec<String> = selenium_containers
            .into_iter()
            .filter(|container| {
                container.name.contains("test") || container.name.contains("selenium")
            })
            .map(|container| container.id)
            .collect();

        if !test_selenium_containers.is_empty() {
            println!(
                "[RUST CLEANUP] Found {} orphaned Selenium test containers",
                test_selenium_containers.len()
            );
            self.remove_containers(&test_selenium_containers, true)
                .await?;
        }

        Ok(())
    }

    /// Clean up test networks
    pub async fn cleanup_test_networks(&self) -> Result<()> {
        println!("[RUST CLEANUP] Cleaning up test networks...");

        let networks = self.list_networks().await?;
        let test_networks: Vec<String> = networks
            .into_iter()
            .filter(|network| {
                network.name.contains("test") || network.name.contains("sortingoffice")
            })
            .map(|network| network.name)
            .collect();

        for network_name in test_networks {
            match self.remove_network(&network_name).await {
                Ok(()) => println!("[RUST CLEANUP] Removed test network: {network_name}"),
                Err(e) => eprintln!(
                    "[RUST CLEANUP] Warning: Failed to remove network {network_name}: {e}"
                ),
            }
        }

        Ok(())
    }

    /// Clean up dangling volumes
    pub async fn cleanup_dangling_volumes(&self) -> Result<()> {
        println!("[RUST CLEANUP] Cleaning up dangling volumes...");

        let volumes = self.list_volumes().await?;
        let dangling_volumes: Vec<String> = volumes
            .into_iter()
            .filter(|volume| volume.name.contains("test") || volume.name.contains("mysql"))
            .map(|volume| volume.name)
            .collect();

        for volume_name in dangling_volumes {
            match self.remove_volume(&volume_name).await {
                Ok(()) => println!("[RUST CLEANUP] Removed dangling volume: {volume_name}"),
                Err(e) => {
                    eprintln!("[RUST CLEANUP] Warning: Failed to remove volume {volume_name}: {e}")
                }
            }
        }

        Ok(())
    }

    /// Comprehensive cleanup of all test resources
    pub async fn cleanup_all_test_resources(&self) -> Result<()> {
        println!("[RUST CLEANUP] Starting comprehensive cleanup of all test resources...");

        // Clean up orphaned containers
        self.cleanup_orphaned_test_containers().await?;

        // Clean up test networks
        self.cleanup_test_networks().await?;

        // Clean up dangling volumes
        self.cleanup_dangling_volumes().await?;

        println!("[RUST CLEANUP] Comprehensive cleanup completed");
        Ok(())
    }
}

/// Convenience function to create a container manager and run cleanup
pub async fn run_rust_cleanup() -> Result<()> {
    let manager = ContainerManager::new();

    match manager.cleanup_all_test_resources().await {
        Ok(()) => {
            println!("[RUST CLEANUP] Cleanup completed successfully");
            Ok(())
        }
        Err(e) => {
            eprintln!("[RUST CLEANUP] Warning: Failed to run cleanup: {e}");
            eprintln!("[RUST CLEANUP] Falling back to shell-based cleanup");
            // Fall back to shell-based cleanup if Rust approach fails
            run_shell_fallback_cleanup().await
        }
    }
}

/// Fallback to shell-based cleanup if Rust approach fails
async fn run_shell_fallback_cleanup() -> Result<()> {
    println!("[SHELL FALLBACK] Running shell-based cleanup...");

    // Basic container cleanup
    let _ = Command::new("docker")
        .args(["container", "prune", "-f"])
        .output();

    // Basic network cleanup
    let _ = Command::new("docker")
        .args(["network", "prune", "-f"])
        .output();

    // Basic volume cleanup
    let _ = Command::new("docker")
        .args(["volume", "prune", "-f"])
        .output();

    println!("[SHELL FALLBACK] Shell-based cleanup completed");
    Ok(())
}
