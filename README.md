# Lukas

Este proyecto es un PoC que tiene por objetivo auydar con las finanzas personales,
especialmente de las personas en Chile. ya que busca conectarse con la banca del país mientras no exista otra opción.

## Ideas

Dentro de las cosas que se quieren probar, están:

- CRUD sobre transacciones
- Conexión con splitwise para unir transacciones
- Conexión con las plataformas bancarias chilenas
- Importe automático de transacciones
- Proyecciones en base a la historia
- Proyecciones en base a estimados del usuario
- Objetivos
- Planificaciones para distintos escenarios

## Especificacion

- [Indice de documentacion](docs/README.md)
- [Resumen de producto](docs/product/overview.md)
- [Casos de uso](docs/product/use-cases.md)
- [Arquitectura](docs/architecture/overview.md)
- [Workflow de desarrollo](docs/engineering/dev-workflow.md)
- [Preguntas abiertas](docs/planning/open-questions.md)

## Stack

### Aplicacion desktop

- Tauri
- Vue 3
- CSS (scss?)
- C#/.NET sidecar
- SQLite local

### Web/SaaS futuro

- Vue o Nuxt
- ASP.NET API
- PostgreSQL
- Object storage para importaciones, snapshots, exports y documentos

### CI/CD

- Planeado. Ver [workflow de desarrollo](docs/engineering/dev-workflow.md) y
  [preguntas abiertas](docs/planning/open-questions.md).
