# RTSyn architecture diagrams

These diagrams describe the released `v1.0.0` architecture at process and
thread level.

- `functional-flow.puml`: startup, command handling, one realtime cycle,
  telemetry, and shutdown as a UML sequence diagram.
- `dependency-view.puml`: static process, thread, queue, storage, and external
  dependency boundaries as a UML deployment/component diagram.

Both views deliberately omit the internal package graph. They describe the
runtime boundary used by an operator: client, HTTP API, SPSC communication,
realtime engine, runtime graph, nodes, and telemetry persistence.

The diagrams were checked against the v1.0.0 implementation and the design
described in the thesis: devices run before and after plugins, only cyclic
plugin-to-plugin edges are delayed, and HTTP/file work remains outside the RT
cycle.

The sources use PlantUML. Render them with a local PlantUML installation:

```bash
plantuml -tsvg docs/architecture/*.puml
```
