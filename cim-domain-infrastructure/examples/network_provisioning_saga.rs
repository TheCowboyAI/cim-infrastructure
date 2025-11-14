//! Network Provisioning Saga Example
//!
//! Demonstrates using cim-domain's Saga infrastructure with a Mealy state machine
//! to coordinate complex infrastructure provisioning workflows.
//!
//! ## Architectural Pattern
//!
//! - **Saga**: Aggregate-of-aggregates (from cim-domain::saga)
//! - **State Machine**: Mealy machine (output depends on state AND input)
//! - **Root**: InfrastructureAggregate acts as saga coordinator
//! - **Participants**: Individual infrastructure entities being provisioned
//!
//! ## Workflow
//!
//! 1. InitiateProvisioning → NetworksDefined
//! 2. NetworksDefined → ResourcesRegistered
//! 3. ResourcesRegistered → InterfacesConfigured
//! 4. InterfacesConfigured → TopologyEstablished
//! 5. TopologyEstablished → ProvisioningComplete
//!
//! Run with:
//! ```bash
//! cargo run --example network_provisioning_saga
//! ```

use cim_domain::saga::{Participant, Saga};
use cim_domain::state_machine::{State, MealyStateTransitions, TransitionInput, TransitionOutput};
use cim_domain::AggregateId as CimAggregateId;
use cim_domain::DomainEvent;

use cim_domain_infrastructure::{
    ComputeResourceSpec, ComputeType, ConnectionSpec, Hostname, InfrastructureAggregate,
    InfrastructureId, InterfaceId, InterfaceSpec, Ipv4Network,
    MessageIdentity, NetworkId, NetworkSpec, NetworkTopologySpec, ResourceCapabilities,
    ResourceId, SystemArchitecture, TopologyId,
};

use std::net::Ipv4Addr;

/// States in the network provisioning saga (Mealy machine)
#[derive(Debug, Clone, PartialEq, Eq)]
enum ProvisioningState {
    /// Initial state - saga initiated
    InitiateProvisioning,
    /// Networks have been defined
    NetworksDefined,
    /// Compute resources registered
    ResourcesRegistered,
    /// Network interfaces configured
    InterfacesConfigured,
    /// Network topology established
    TopologyEstablished,
    /// Provisioning complete (terminal state)
    ProvisioningComplete,
    /// Provisioning failed (terminal state)
    ProvisioningFailed { reason: String },
}

impl State for ProvisioningState {
    fn name(&self) -> &'static str {
        match self {
            Self::InitiateProvisioning => "InitiateProvisioning",
            Self::NetworksDefined => "NetworksDefined",
            Self::ResourcesRegistered => "ResourcesRegistered",
            Self::InterfacesConfigured => "InterfacesConfigured",
            Self::TopologyEstablished => "TopologyEstablished",
            Self::ProvisioningComplete => "ProvisioningComplete",
            Self::ProvisioningFailed { .. } => "ProvisioningFailed",
        }
    }

    fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::ProvisioningComplete | Self::ProvisioningFailed { .. }
        )
    }
}

/// Inputs that drive state transitions
#[derive(Debug, Clone)]
enum ProvisioningInput {
    /// Networks defined successfully
    NetworksCreated { count: usize },
    /// Resources registered successfully
    ResourcesProvisioned { count: usize },
    /// Interfaces configured successfully
    InterfacesAttached { count: usize },
    /// Topology established successfully
    ConnectionsEstablished { count: usize },
    /// Operation completed successfully
    OperationSuccess,
    /// Operation failed
    #[allow(dead_code)]
    OperationFailed { error: String },
}

impl TransitionInput for ProvisioningInput {
    fn description(&self) -> String {
        match self {
            Self::NetworksCreated { count } => format!("Created {} networks", count),
            Self::ResourcesProvisioned { count } => format!("Provisioned {} resources", count),
            Self::InterfacesAttached { count } => format!("Attached {} interfaces", count),
            Self::ConnectionsEstablished { count } => format!("Established {} connections", count),
            Self::OperationSuccess => "Operation succeeded".to_string(),
            Self::OperationFailed { error } => format!("Operation failed: {}", error),
        }
    }
}

/// Outputs from state transitions
#[derive(Debug, Clone)]
struct ProvisioningOutput {
    message: String,
    #[allow(dead_code)]
    events_generated: usize,
}

impl TransitionOutput for ProvisioningOutput {
    fn to_events(&self) -> Vec<Box<dyn DomainEvent>> {
        // In a real implementation, these would be infrastructure events
        vec![]
    }
}

/// Mealy state machine for network provisioning saga
impl MealyStateTransitions for ProvisioningState {
    type Input = ProvisioningInput;
    type Output = ProvisioningOutput;

    fn can_transition_to(&self, target: &Self, input: &Self::Input) -> bool {
        use ProvisioningInput::*;
        use ProvisioningState::*;

        match (self, target, input) {
            // Happy path transitions
            (InitiateProvisioning, NetworksDefined, NetworksCreated { .. }) => true,
            (NetworksDefined, ResourcesRegistered, ResourcesProvisioned { .. }) => true,
            (ResourcesRegistered, InterfacesConfigured, InterfacesAttached { .. }) => true,
            (InterfacesConfigured, TopologyEstablished, ConnectionsEstablished { .. }) => true,
            (TopologyEstablished, ProvisioningComplete, OperationSuccess) => true,

            // Failure transitions from any non-terminal state
            (state, ProvisioningFailed { .. }, OperationFailed { .. }) if !state.is_terminal() => {
                true
            }

            _ => false,
        }
    }

    fn valid_transitions(&self, input: &Self::Input) -> Vec<Self> {
        use ProvisioningInput::*;
        use ProvisioningState::*;

        match (self, input) {
            (InitiateProvisioning, NetworksCreated { .. }) => vec![NetworksDefined],
            (NetworksDefined, ResourcesProvisioned { .. }) => vec![ResourcesRegistered],
            (ResourcesRegistered, InterfacesAttached { .. }) => vec![InterfacesConfigured],
            (InterfacesConfigured, ConnectionsEstablished { .. }) => vec![TopologyEstablished],
            (TopologyEstablished, OperationSuccess) => vec![ProvisioningComplete],
            (state, OperationFailed { error }) if !state.is_terminal() => {
                vec![ProvisioningFailed {
                    reason: error.clone(),
                }]
            }
            _ => vec![],
        }
    }

    fn transition_output(&self, target: &Self, input: &Self::Input) -> Self::Output {
        ProvisioningOutput {
            message: format!(
                "Transitioned from {} to {} via {}",
                self.name(),
                target.name(),
                input.description()
            ),
            events_generated: 1,
        }
    }
}

/// Network Provisioning Saga
///
/// Coordinates infrastructure provisioning across multiple aggregates
struct NetworkProvisioningSaga {
    /// The saga coordination structure
    saga: Saga,
    /// Current state in the provisioning workflow
    state: ProvisioningState,
    /// Infrastructure aggregate (saga root)
    infrastructure: InfrastructureAggregate,
}

impl NetworkProvisioningSaga {
    /// Create a new network provisioning saga
    fn new(infrastructure_id: InfrastructureId) -> Self {
        // Saga root is the infrastructure aggregate
        let root = Participant {
            id: CimAggregateId::new(),
            domain: Some("infrastructure".to_string()),
        };

        Self {
            saga: Saga::new(root),
            state: ProvisioningState::InitiateProvisioning,
            infrastructure: InfrastructureAggregate::new(infrastructure_id),
        }
    }

    /// Add a participant to the saga
    fn with_participant(mut self, domain: &str) -> Self {
        let participant = Participant {
            id: CimAggregateId::new(),
            domain: Some(domain.to_string()),
        };
        self.saga = self.saga.with_participant(participant);
        self
    }

    /// Process a transition input
    fn process_input(&mut self, input: ProvisioningInput) -> Result<ProvisioningOutput, String> {
        // Check if transition is valid
        let valid_targets = self.state.valid_transitions(&input);
        if valid_targets.is_empty() {
            return Err(format!(
                "No valid transitions from {} with input {}",
                self.state.name(),
                input.description()
            ));
        }

        let new_state = valid_targets[0].clone();

        // Get transition output
        let output = self
            .state
            .transition_output(&new_state, &input);

        // Advance vector clock (track causal ordering)
        self.saga = self.saga.tick("provisioning-coordinator");

        // Update state
        self.state = new_state;

        Ok(output)
    }

    /// Execute the complete provisioning workflow
    fn execute_provisioning(&mut self) -> Result<(), String> {
        let identity = MessageIdentity::new_root();

        println!("🚀 Starting Network Provisioning Saga");
        println!("   State: {}\n", self.state.name());

        // Step 1: Define networks
        println!("📡 Step 1: Defining networks...");
        let frontend_net = NetworkSpec {
            id: NetworkId::new("frontend").unwrap(),
            name: "Frontend Network".to_string(),
            cidr_v4: Some(Ipv4Network::new(Ipv4Addr::new(192, 168, 1, 0), 24).unwrap()),
            cidr_v6: None,
        };

        let backend_net = NetworkSpec {
            id: NetworkId::new("backend").unwrap(),
            name: "Backend Network".to_string(),
            cidr_v4: Some(Ipv4Network::new(Ipv4Addr::new(10, 0, 0, 0), 24).unwrap()),
            cidr_v6: None,
        };

        self.infrastructure
            .handle_define_network(frontend_net, &identity)
            .map_err(|e| e.to_string())?;
        self.infrastructure
            .handle_define_network(backend_net, &identity)
            .map_err(|e| e.to_string())?;

        let output = self.process_input(ProvisioningInput::NetworksCreated { count: 2 })?;
        println!("   ✓ {}", output.message);
        println!("   State: {}\n", self.state.name());

        // Step 2: Register compute resources
        println!("💻 Step 2: Registering compute resources...");
        let web_server = ComputeResourceSpec {
            id: ResourceId::new("web-01").unwrap(),
            resource_type: ComputeType::VirtualMachine,
            hostname: Hostname::new("web-01").unwrap(),
            system: SystemArchitecture::x86_64_linux(),
            capabilities: ResourceCapabilities::default(),
        };

        let app_server = ComputeResourceSpec {
            id: ResourceId::new("app-01").unwrap(),
            resource_type: ComputeType::Container,
            hostname: Hostname::new("app-01").unwrap(),
            system: SystemArchitecture::x86_64_linux(),
            capabilities: ResourceCapabilities::default(),
        };

        self.infrastructure
            .handle_register_compute_resource(web_server, &identity)
            .map_err(|e| e.to_string())?;
        self.infrastructure
            .handle_register_compute_resource(app_server, &identity)
            .map_err(|e| e.to_string())?;

        let output = self.process_input(ProvisioningInput::ResourcesProvisioned { count: 2 })?;
        println!("   ✓ {}", output.message);
        println!("   State: {}\n", self.state.name());

        // Step 3: Configure interfaces
        println!("🔌 Step 3: Configuring network interfaces...");
        let web_interface = InterfaceSpec {
            id: InterfaceId::new("web-eth0").unwrap(),
            resource_id: ResourceId::new("web-01").unwrap(),
            network_id: Some(NetworkId::new("frontend").unwrap()),
            addresses: vec!["192.168.1.10".parse().unwrap()],
        };

        let app_interface = InterfaceSpec {
            id: InterfaceId::new("app-eth0").unwrap(),
            resource_id: ResourceId::new("app-01").unwrap(),
            network_id: Some(NetworkId::new("backend").unwrap()),
            addresses: vec!["10.0.0.10".parse().unwrap()],
        };

        self.infrastructure
            .handle_add_interface(web_interface, &identity)
            .map_err(|e| e.to_string())?;
        self.infrastructure
            .handle_add_interface(app_interface, &identity)
            .map_err(|e| e.to_string())?;

        let output = self.process_input(ProvisioningInput::InterfacesAttached { count: 2 })?;
        println!("   ✓ {}", output.message);
        println!("   State: {}\n", self.state.name());

        // Step 4: Establish topology
        println!("🔗 Step 4: Establishing network topology...");
        let topology = NetworkTopologySpec {
            topology_id: TopologyId::new(),
            networks: vec![],
            connections: vec![ConnectionSpec {
                from_resource: ResourceId::new("web-01").unwrap(),
                from_interface: InterfaceId::new("web-eth0").unwrap(),
                to_resource: ResourceId::new("app-01").unwrap(),
                to_interface: InterfaceId::new("app-eth0").unwrap(),
            }],
        };

        self.infrastructure
            .handle_define_network_topology(topology, &identity)
            .map_err(|e| e.to_string())?;

        let output = self.process_input(ProvisioningInput::ConnectionsEstablished { count: 1 })?;
        println!("   ✓ {}", output.message);
        println!("   State: {}\n", self.state.name());

        // Step 5: Complete provisioning
        println!("✅ Step 5: Completing provisioning...");
        let output = self.process_input(ProvisioningInput::OperationSuccess)?;
        println!("   ✓ {}", output.message);
        println!("   State: {}\n", self.state.name());

        println!("🎉 Network Provisioning Saga Complete!");
        println!("\n📊 Saga Statistics:");
        println!("   Root: {:?}", self.saga.root.domain);
        println!("   Participants: {}", self.saga.participants.len());
        println!("   Vector Clock: {:?}", self.saga.clock);
        println!("   Final State: {}", self.state.name());

        Ok(())
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("═══════════════════════════════════════════════════════");
    println!("  Network Provisioning Saga Example");
    println!("  Demonstrates: Saga as Aggregate-of-Aggregates");
    println!("               + Mealy State Machine");
    println!("═══════════════════════════════════════════════════════\n");

    let mut saga = NetworkProvisioningSaga::new(InfrastructureId::new())
        .with_participant("network")
        .with_participant("compute")
        .with_participant("topology");

    saga.execute_provisioning()?;

    println!("\n💡 Key Architectural Points:");
    println!("   1. Saga composes multiple aggregates (networks, resources, topology)");
    println!("   2. Mealy state machine drives transitions based on state + input");
    println!("   3. Vector clock tracks causal ordering across participants");
    println!("   4. Infrastructure aggregate acts as saga root/coordinator");
    println!("   5. No new module needed - uses cim-domain::saga!");

    Ok(())
}
