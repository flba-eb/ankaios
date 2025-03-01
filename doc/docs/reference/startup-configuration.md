# Startup configuration
In the Ankaios system it is mandatory to specify all the [nodes](./glossary.md#node) and [workloads](./glossary.md#workload) that are going to be run. Currently the startup configuration is provided as a file which is in YAML file format and can be passed to the Ankaios server through a command line argument. Depending on the demands towards Ankaios, the startup configuration can later be provided in a different way.

## Configuration structure
The startup configuration is composed of a list of workload specifications within the `workloads` object.
A workload specification must contain the following information:

* `workload name`_(via field key)_, specify the workload name to identify the workload in the Ankaios system.
* `runtime`, specify the type of the runtime. Currently supported value is `podman`.
* `agent`, specify the name of the owning agent which is going to execute the workload.
* `restart`, specify if the workload shall be restarted when it exits. Currently not implemented.
* `updateStrategy`, specify the update strategy which can be one of the following values:
    * `UNSPECIFIED`
    * `AT_LEAST_ONCE`
    * `AT_MOST_ONCE`
* `accessRights`, specify lists of access rules for `allow` and `deny` (currently not implemented and shall be set to empty list for both).
* `tags`, specify a list of `key` `value`  pairs.
* `runtimeConfig`, specify as a _string_ the configuration for the [runtime](./glossary.md#runtime) whose configuration structure is specific for each runtime, e.g., for `podman` runtime the [PodmanRuntimeConfig](#podmanruntimeconfig) is used.

Example `startup-config.yaml` file:
```yaml
workloads:
  nginx: # this is used as the workload name which is 'nginx'
    runtime: podman
    agent: agent_A
    restart: true
    updateStrategy: AT_MOST_ONCE
    accessRights:
      allow: []
      deny: []
    tags:
      - key: owner
        value: Ankaios team
    runtimeConfig: |
      image: docker.io/nginx:latest
      ports:
      - containerPort: 80
        hostPort: 8081
  api_sample: # this is used as the workload name which is 'api_sample'
    runtime: podman
    agent: agent_A
    restart: true
    updateStrategy: AT_MOST_ONCE
    accessRights:
      allow: []
      deny: []
    tags:
      - key: owner
        value: Ankaios team
    runtimeConfig: |
      image: ankaios_workload_api_example
```

### PodmanRuntimeConfig
The runtime configuration for the `podman` runtime is specified as followed:
```text
image: <string>                   # Image repository or image id
command: <vector of strings>      # Entrypoint array. Not executed in a shell.
args: <vector of strings>         # Arguments to the entrypoint. The container image's CMD is used if this is not provided.
env:  <vector of name/value pair>  # Key/value pairs provided as environment variables in the container
ports: <vector of port mappings>  # List of ports to be exposed
remove: <true|false>              # Specify whether the container shall be removed after exited
```

