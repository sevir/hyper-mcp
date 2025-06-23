# maven

A plugin that fetches the dependencies of a Maven package (from Maven Central) given its group, artifact, and version.

## What it does

Given a Maven package (groupId, artifactId, version), fetches its POM file from Maven Central and returns its dependencies as JSON.

## Usage

Call with:
```json
{
  "plugins": [
    {
      "name": "mvn_fetch_deps",
      "path": "oci://ghcr.io/tuananh/maven-plugin:latest",
      "runtime_config": {
        "allowed_hosts": ["repo1.maven.org"]
      }
    }
  ]
}
```

### Example input

```json
{
  "name": "mvn_fetch_deps",
  "arguments": {
    "group": "org.springframework.boot",
    "artifact": "spring-boot-starter-web",
    "version": "3.5.0"
  }
}
```

### Example output

```json
{
  "dependencies": [
    {
      "groupId": "org.springframework.boot",
      "artifactId": "spring-boot-starter",
      "version": "3.5.0",
      "scope": "compile"
    },
    {
      "groupId": "org.springframework.boot",
      "artifactId": "spring-boot-starter-json",
      "version": "3.5.0",
      "scope": "compile"
    }
    // ...
  ]
}
```

Return the list of dependencies of the given Maven package.