# Pricing Estimation

**Capability:** Show estimated costs before launching

---

## ADDED Requirements

### Requirement: Cost Estimation

The app SHALL display the estimated hourly cost for both spot and on-demand pricing before VM launch.

#### Scenario: Display hourly cost for spot and on-demand

- **WHEN** the user has configured a VM (machine type, GPU, provisioning model) and the launch panel is visible
- **THEN** the app SHALL display both the spot hourly rate and the on-demand hourly rate for the configured VM

#### Scenario: Cost updates on configuration change

- **WHEN** the user changes any configuration parameter (machine type, GPU type, GPU count)
- **THEN** the app SHALL recalculate and update the displayed spot and on-demand hourly costs immediately

#### Scenario: No GPU cost when GPU is not selected

- **WHEN** the user configures an N1 machine type without a GPU
- **THEN** the app SHALL display the hourly cost for vCPU and memory only, with no GPU cost component

---

### Requirement: Pricing Breakdown

The app SHALL show a breakdown of costs by component: vCPU, memory, and GPU (when applicable).

#### Scenario: Breakdown with GPU

- **WHEN** the user configures an N1 machine type with 4x T4 GPUs
- **THEN** the app SHALL display separate line items for vCPU cost, memory cost, and GPU cost (4x T4), each showing the per-hour rate

#### Scenario: Breakdown without GPU

- **WHEN** the user configures a machine type with no GPU
- **THEN** the app SHALL display line items for vCPU cost and memory cost only

#### Scenario: Breakdown for A2/A3 machine types

- **WHEN** the user configures an A2 or A3 machine type
- **THEN** the app SHALL display the total hourly rate for the machine type as a single line item, since GPU is bundled

---

### Requirement: Monthly Projection

The app SHALL show the projected monthly cost based on 730 hours per month.

#### Scenario: Monthly projection calculation

- **WHEN** the estimated hourly cost is $2.50/hr for spot
- **THEN** the app SHALL display the projected monthly cost as $1,825.00 (2.50 x 730) for spot

#### Scenario: Monthly projection for both provisioning models

- **WHEN** the pricing panel is visible
- **THEN** the app SHALL display monthly projections for both spot and on-demand pricing

---

### Requirement: Embedded Pricing

The app SHALL use an embedded pricing table for cost calculations. Pricing data MAY be refreshed from the Cloud Billing API as a future enhancement.

#### Scenario: Pricing available offline

- **WHEN** the app is launched without network access to the Cloud Billing API
- **THEN** the app SHALL still display pricing estimates using the embedded pricing table

#### Scenario: Embedded table covers configured machine types

- **WHEN** the user selects any machine type or GPU type offered by the app's configuration UI
- **THEN** the embedded pricing table SHALL contain pricing data for that selection and return a valid estimate

#### Scenario: Unknown machine type fallback

- **WHEN** a machine type is not found in the embedded pricing table
- **THEN** the app SHALL display a message indicating that pricing is unavailable for the selected configuration
