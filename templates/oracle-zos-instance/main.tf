# ZOS Oracle Cloud Stack Template
# Deploys rate-limited libp2p nodes for solfunmeme network
# AGPL-3.0 License

variable "compartment_id" {
  description = "Oracle Cloud compartment OCID"
  type        = string
  default     = "ocid1.tenancy.oc1..aaaaaaaapxfkcjaczqslvnbekbqq2eefxgwx7kqbakvddhzaaiym62vmt5la"
}

variable "vcn_id" {
  description = "VCN OCID for networking"
  type        = string
}

variable "subnet_id" {
  description = "Subnet OCID for instance placement"
  type        = string
}

variable "availability_domain" {
  description = "Availability domain for instance"
  type        = string
}

variable "zos_account" {
  description = "ZOS network account name"
  type        = string
  default     = "solfunmeme"
}

variable "rate_limit_connections" {
  description = "Rate limit connections per minute per IP"
  type        = number
  default     = 10
}

# Free tier Oracle Linux instance
resource "oci_core_instance" "zos_node" {
  availability_domain = var.availability_domain
  compartment_id      = var.compartment_id
  display_name        = "zos-${var.zos_account}-node"
  shape              = "VM.Standard.E2.1.Micro"  # Always Free

  source_details {
    source_id   = "ocid1.image.oc1.iad.aaaaaaaav3mbu4obi2rusqeswiva6pwnsbxpuacuu5ifkpqyne5yco4my33a"
    source_type = "image"
  }

  create_vnic_details {
    subnet_id        = var.subnet_id
    display_name     = "zos-vnic"
    assign_public_ip = true
    nsg_ids          = [oci_core_network_security_group.zos_public.id]
  }

  metadata = {
    user_data = base64encode(templatefile("${path.module}/zos-boot-script.sh", {
      zos_account = var.zos_account
      rate_limit  = var.rate_limit_connections
    }))
  }

  freeform_tags = {
    "ZOS-Account" = var.zos_account
    "ZOS-Type"    = "libp2p-node"
    "ZOS-Tier"    = "free"
  }
}

# Network security group for ZOS public access
resource "oci_core_network_security_group" "zos_public" {
  compartment_id = var.compartment_id
  vcn_id         = var.vcn_id
  display_name   = "zos-${var.zos_account}-public"

  freeform_tags = {
    "ZOS-Account" = var.zos_account
  }
}

# Allow public libp2p access on port 4001
resource "oci_core_network_security_group_security_rule" "zos_libp2p_tcp" {
  network_security_group_id = oci_core_network_security_group.zos_public.id
  direction                 = "INGRESS"
  protocol                  = "6"  # TCP

  source      = "0.0.0.0/0"
  source_type = "CIDR_BLOCK"

  tcp_options {
    destination_port_range {
      min = 4001
      max = 4001
    }
  }
}

# Allow public libp2p access on port 4001 UDP (QUIC)
resource "oci_core_network_security_group_security_rule" "zos_libp2p_udp" {
  network_security_group_id = oci_core_network_security_group.zos_public.id
  direction                 = "INGRESS"
  protocol                  = "17"  # UDP

  source      = "0.0.0.0/0"
  source_type = "CIDR_BLOCK"

  udp_options {
    destination_port_range {
      min = 4001
      max = 4001
    }
  }
}

# Outputs for network registration
output "zos_node_info" {
  description = "ZOS node information for network registration"
  value = {
    account         = var.zos_account
    instance_id     = oci_core_instance.zos_node.id
    public_ip       = oci_core_instance.zos_node.public_ip
    private_ip      = oci_core_instance.zos_node.private_ip
    public_endpoint = "${oci_core_instance.zos_node.public_ip}:4001"
    rate_limit      = "${var.rate_limit_connections}/minute"
    tier           = "free"
  }
}

output "zos_public_ip" {
  description = "Public IP for libp2p connections"
  value       = oci_core_instance.zos_node.public_ip
}

output "zos_bootstrap_url" {
  description = "Bootstrap URL for other nodes to connect"
  value       = "https://${oci_core_instance.zos_node.public_ip}:4001/p2p/${var.zos_account}"
}
