## Spacejam Sync

This module contains the state transition logic for the Spacejam protocol.

## Dependency Graph

> [!NOTE]
> Only shared dependencies will be shown in the graph.

```mermaid
flowchart LR
   %% The new block header
   %% H(H)
   %% H --> TAU_PRIME
   %% H --> BETA_DAGGER
   %% H --> GAMMA_PRIME
   %% H --> ETA_PRIME
   %% H --> KAPPA_PRIME
   %% H --> LAMBDA_PRIME
   %% H --> OMEGA_PRIME

   %% The accumulation-commitment
   %% C(C)
   %% C --> RHO_PRIME

   %% Extrinsic guarantee
   %% EG[[E_G]]
   %% EG --> BETA_PRIME
   %% EG --> RHO_PRIME

   %% Extrinsic Ticket
   %% ET[[E_T]]
   %% ET --> GAMMA_PRIME
   %% ET --> PI_PRIME

   %% The old timeslot index
   %% TAU((τ))
   %% TAU --> GAMMA_PRIME
   %% TAU --> ETA_PRIME
   %% TAU --> KAPPA_PRIME
   %% TAU --> LAMBDA_PRIME
   %% TAU --> PI_PRIME


   %% The old block history
   %% BETA((β))
   %% BETA --> BETA_DAGGER

   %% The updated TIMESLOTchived validators
   LAMBDA_PRIME["λ' (H)"]

   %% An intermediate value for block history
   BETA_DAGGER["β† (H, β)"]
   BETA_DAGGER --> BETA_PRIME

   %% The updated block history
   BETA_PRIME["β' (H, E_G)"]

   %% The old validator state
   %% GAMMA((γ))
   %% GAMMA --> GAMMA_PRIME
   %% GAMMA --> KAPPA_PRIME

   %% The updated validator state
   GAMMA_PRIME[γ']

   %% The old entropy pool
   %% ETA((η))
   %% ETA --> ETA_PRIME

   %% The updated entropy pool
   ETA_PRIME[η']
   ETA_PRIME --> GAMMA_PRIME

   %% The updated TIMESLOTchived validators
   KAPPA_PRIME[κ']
   KAPPA_PRIME --> GAMMA_PRIME
   KAPPA_PRIME --> PI_PRIME

   %% An intermediate value for work reports
   RHO_DAGGER[ρ†]
   RHO_DAGGER --> RHO_PRIME

   %%  Another intermediate value for work reports
   RHO_PRIME[ρ‡]
   RHO_PRIME --> KAPPA_PRIME

   %% The new timeslot index
   TAU_PRIME[τ']
   TAU_PRIME --> RHO_PRIME
   TAU_PRIME --> DELTA_PRIME

   %% The updated judgements
   PSI_PRIME[ψ']
   PSI_PRIME --> GAMMA_PRIME

   %% The updated report state
   RHO_PRIME[ρ']
   RHO_PRIME --> WORK_REPORTS

   %% The set of work-reports ready for accumulation
   WORK_REPORTS[W*]
   WORK_REPORTS --> ACCUMULATION

   %% accumulation
   ACCUMULATION[(ϑ', ξ', δ‡, χ', ι', φ', C)]
   ACCUMULATION --> PSI_PRIME
   ACCUMULATION --> BETA_PRIME
   ACCUMULATION --> OMEGA_PRIME

   %% The updated service state
   DELTA_PRIME[δ']

   %% The updated authorization pool
   OMEGA_PRIME[α']

   %% The updated statistics state
   PI_PRIME[π']

   %% computation orders
   subgraph "first"
   LAMBDA_PRIME
   RHO_DAGGER
   TAU_PRIME
   BETA_DAGGER
   ETA_PRIME
   end

   subgraph "second"
   DELTA_PRIME
   RHO_PRIME
   end

   subgraph "third"
   KAPPA_PRIME
   WORK_REPORTS
   end

   subgraph "fourth"
   PI_PRIME
   ACCUMULATION
   end

   subgraph "fifth"
   BETA_PRIME
   PSI_PRIME
   OMEGA_PRIME
   end

   subgraph "sixth"
   GAMMA_PRIME
   end
```
