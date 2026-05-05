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

- [Casos de uso](docs/use-cases.md)
- [Backlog de discusion](docs/phase-2/discussion-backlog.md)
- [Requisitos tecnicos por discutir](docs/phase-2/technical_requirements.md)
- [Detalles tecnicos por discutir](docs/phase-2/technical-details.md)
- [Detalles de implementacion por discutir](docs/phase-2/implementation-details.md)

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

- TBD
