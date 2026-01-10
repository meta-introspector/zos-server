# ZOS Oracle Cloud Stack - Complete Configuration
# Deploys ZOS server on Oracle Cloud free tier
# AGPL-3.0 License

terraform {
  required_version = ">= 1.0"
  required_providers {
    oci = {
      source  = "oracle/oci"
      version = "~> 5.0"
    }
  }
}

# Configure Oracle Cloud provider
provider "oci" {
  # Authentication via OCI CLI config or environment variables
  # OCI_TENANCY_OCID, OCI_USER_OCID, OCI_FINGERPRINT, OCI_PRIVATE_KEY_PATH
}

# Data sources for Oracle Cloud resources
data "oci_identity_availability_domains" "ads" {
  compartment_id = var.compartment_id
}

data "oci_core_images" "oracle_linux" {
  compartment_id           = var.compartment_id
  operating_system         = "Oracle Linux"
  operating_system_version = "8"
  shape                    = "VM.Standard.E2.1.Micro"
  sort_by                  = "TIMECREATED"
  sort_order              = "DESC"
}

# Variables
variable "compartment_id" {
  description = "Oracle Cloud compartment OCID"
  type        = string
}

variable "zos_domain" {
  description = "Domain name for ZOS server"
  type        = string
  default     = "node1.solfunmeme.com"
}

variable "ssh_public_key" {
  description = "SSH public key for instance access"
  type        = string
}

# Create VCN if not provided
resource "oci_core_vcn" "zos_vcn" {
  compartment_id = var.compartment_id
  display_name   = "zos-vcn"
  cidr_blocks    = ["10.0.0.0/16"]
  dns_label      = "zosvcn"

  freeform_tags = {
    "Project" = "ZOS"
  }
}

# Internet Gateway
resource "oci_core_internet_gateway" "zos_igw" {
  compartment_id = var.compartment_id
  vcn_id         = oci_core_vcn.zos_vcn.id
  display_name   = "zos-igw"
  enabled        = true
}

# Route Table
resource "oci_core_default_route_table" "zos_rt" {
  manage_default_resource_id = oci_core_vcn.zos_vcn.default_route_table_id
  display_name               = "zos-route-table"

  route_rules {
    destination       = "0.0.0.0/0"
    destination_type  = "CIDR_BLOCK"
    network_entity_id = oci_core_internet_gateway.zos_igw.id
  }
}

# Public Subnet
resource "oci_core_subnet" "zos_public_subnet" {
  compartment_id      = var.compartment_id
  vcn_id              = oci_core_vcn.zos_vcn.id
  display_name        = "zos-public-subnet"
  cidr_block          = "10.0.1.0/24"
  dns_label           = "zospublic"
  security_list_ids   = [oci_core_security_list.zos_public_sl.id]
  route_table_id      = oci_core_vcn.zos_vcn.default_route_table_id
  dhcp_options_id     = oci_core_vcn.zos_vcn.default_dhcp_options_id
}

# Security List
resource "oci_core_security_list" "zos_public_sl" {
  compartment_id = var.compartment_id
  vcn_id         = oci_core_vcn.zos_vcn.id
  display_name   = "zos-public-security-list"

  # Allow SSH
  ingress_security_rules {
    protocol = "6" # TCP
    source   = "0.0.0.0/0"
    tcp_options {
      min = 22
      max = 22
    }
  }

  # Allow HTTP
  ingress_security_rules {
    protocol = "6" # TCP
    source   = "0.0.0.0/0"
    tcp_options {
      min = 80
      max = 80
    }
  }

  # Allow HTTPS
  ingress_security_rules {
    protocol = "6" # TCP
    source   = "0.0.0.0/0"
    tcp_options {
      min = 443
      max = 443
    }
  }

  # Allow ZOS HTTP server
  ingress_security_rules {
    protocol = "6" # TCP
    source   = "0.0.0.0/0"
    tcp_options {
      min = 8080
      max = 8080
    }
  }

  # Allow LibP2P TCP
  ingress_security_rules {
    protocol = "6" # TCP
    source   = "0.0.0.0/0"
    tcp_options {
      min = 4001
      max = 4001
    }
  }

  # Allow LibP2P UDP (QUIC)
  ingress_security_rules {
    protocol = "17" # UDP
    source   = "0.0.0.0/0"
    udp_options {
      min = 4001
      max = 4001
    }
  }

  # Allow all outbound
  egress_security_rules {
    protocol    = "all"
    destination = "0.0.0.0/0"
  }
}

# ZOS Server Instance
resource "oci_core_instance" "zos_server" {
  availability_domain = data.oci_identity_availability_domains.ads.availability_domains[0].name
  compartment_id      = var.compartment_id
  display_name        = "zos-server"
  shape               = "VM.Standard.E2.1.Micro" # Always Free

  shape_config {
    ocpus         = 1
    memory_in_gbs = 1
  }

  source_details {
    source_id   = data.oci_core_images.oracle_linux.images[0].id
    source_type = "image"
  }

  create_vnic_details {
    subnet_id        = oci_core_subnet.zos_public_subnet.id
    display_name     = "zos-vnic"
    assign_public_ip = true
  }

  metadata = {
    ssh_authorized_keys = var.ssh_public_key
    user_data = base64encode(templatefile("${path.module}/zos-boot-script.sh", {
      zos_domain = var.zos_domain
    }))
  }

  freeform_tags = {
    "Project" = "ZOS"
    "Type"    = "server"
    "Tier"    = "free"
  }
}

# Outputs
output "zos_server_info" {
  description = "ZOS server connection information"
  value = {
    instance_id = oci_core_instance.zos_server.id
    public_ip   = oci_core_instance.zos_server.public_ip
    private_ip  = oci_core_instance.zos_server.private_ip
    ssh_command = "ssh opc@${oci_core_instance.zos_server.public_ip}"
    http_url    = "http://${oci_core_instance.zos_server.public_ip}:8080"
    https_url   = "https://${var.zos_domain}"
  }
}

output "public_ip" {
  description = "Public IP address of ZOS server"
  value       = oci_core_instance.zos_server.public_ip
}

output "dns_setup" {
  description = "DNS setup instructions"
  value = {
    domain     = var.zos_domain
    ip_address = oci_core_instance.zos_server.public_ip
    record     = "A record: ${var.zos_domain} -> ${oci_core_instance.zos_server.public_ip}"
  }
}
