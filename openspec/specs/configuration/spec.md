# Configuration

**Capability:** Persist and manage settings

---

## ADDED Requirements

### Requirement: Project/Zone Config

The user SHALL be able to set the GCP project name and zone.

#### Scenario: Set project and zone

- **WHEN** the user enters a project name (e.g., `my-gcp-project`) and zone (e.g., `us-central1-a`) in the configuration panel
- **THEN** the app SHALL store these values and use them for all subsequent gcloud CLI commands

#### Scenario: Missing project or zone

- **WHEN** the user attempts to list disks or create a VM without having configured a project or zone
- **THEN** the app SHALL display an error indicating that project and zone must be configured first

#### Scenario: Change project

- **WHEN** the user changes the configured project
- **THEN** the app SHALL clear the current disk list and VM states, and reload disks from the new project

---

### Requirement: Auth Config

The app SHALL support gcloud CLI auth (default) and optional service account key file via the `CLOUDSDK_AUTH_CREDENTIAL_FILE_OVERRIDE` environment variable.

#### Scenario: Default gcloud CLI auth

- **WHEN** the user has not configured a service account key file
- **THEN** the app SHALL use the default gcloud CLI authentication (i.e., the credentials from `gcloud auth login`)

#### Scenario: Service account key file

- **WHEN** the user provides a path to a service account JSON key file
- **THEN** the app SHALL set `CLOUDSDK_AUTH_CREDENTIAL_FILE_OVERRIDE` to that path in the environment of all gcloud subprocess invocations

#### Scenario: Invalid key file path

- **WHEN** the user provides a service account key file path that does not exist
- **THEN** the app SHALL display an error indicating the file was not found and SHALL NOT persist the invalid path

---

### Requirement: Auth Status

The app SHALL display the current authentication status.

#### Scenario: Display authenticated account

- **WHEN** gcloud CLI auth is active and `gcloud auth list --format=json` returns an active account
- **THEN** the app SHALL display the active account email in the configuration panel

#### Scenario: No active auth

- **WHEN** no gcloud account is authenticated and no service account key is configured
- **THEN** the app SHALL display a warning indicating that authentication is required

#### Scenario: Service account auth active

- **WHEN** a service account key file is configured and valid
- **THEN** the app SHALL display the service account email from the key file as the active identity

---

### Requirement: Persistence

The app SHALL persist all preferences to a JSON config file in the Tauri app data directory.

#### Scenario: Save configuration on change

- **WHEN** the user changes any configuration value (project, zone, auth, defaults)
- **THEN** the app SHALL write the updated configuration to a JSON file in the Tauri app data directory

#### Scenario: Load configuration on startup

- **WHEN** the app launches and a config file exists in the Tauri app data directory
- **THEN** the app SHALL read the config file and populate all settings from the persisted values

#### Scenario: First launch with no config file

- **WHEN** the app launches and no config file exists
- **THEN** the app SHALL use built-in defaults and prompt the user to configure project and zone

#### Scenario: Corrupted config file

- **WHEN** the app launches and the config file contains invalid JSON
- **THEN** the app SHALL log a warning, discard the corrupted file, and start with built-in defaults

---

### Requirement: Defaults

The app SHALL remember the user's default machine type, GPU type, GPU count, and spot preference.

#### Scenario: Remember default machine type

- **WHEN** the user sets a default machine type of `n1-standard-8`
- **THEN** the app SHALL pre-select `n1-standard-8` as the machine type whenever the VM configuration panel is opened

#### Scenario: Remember default GPU configuration

- **WHEN** the user sets default GPU type to `nvidia-tesla-t4` and count to `2`
- **THEN** the app SHALL pre-populate these GPU values when configuring VMs with N1 machine types

#### Scenario: Remember spot preference

- **WHEN** the user changes the default provisioning model to on-demand
- **THEN** the app SHALL default to on-demand provisioning for all subsequent VM configurations until changed

#### Scenario: Defaults persist across restarts

- **WHEN** the user sets defaults and restarts the app
- **THEN** the app SHALL load the previously saved defaults from the config file and apply them
