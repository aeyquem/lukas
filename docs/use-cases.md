# Especificacion de casos de uso

Este documento baja las ideas iniciales del README a casos de uso mas concretos
para el PoC de Presupuesto. El foco inicial es ayudar con finanzas personales en
Chile, partiendo con flujos manuales y dejando preparada una capa de integracion
para Open Finance y APIs publicas de instituciones financieras.

## Principios del producto

- Chile primero: montos en CLP, instituciones locales, formatos locales y soporte
  para cuentas bancarias, tarjetas y plataformas de inversion usadas en Chile.
- Multimoneda desde el modelo: CLP sera la moneda principal para reportes, pero
  cuentas y transacciones deben conservar su moneda real cuando vengan en USD u
  otra moneda.
- Moneda global configurable: el usuario o workspace debe tener una moneda base
  para reportes. Las cuentas la heredan por defecto, salvo que el usuario defina
  una moneda distinta para una cuenta especifica.
- Manual antes que automatico: cualquier dato importado debe poder revisarse,
  corregirse y complementar el flujo manual.
- Integraciones desacopladas: la aplicacion no debe depender directamente de un
  banco, fintech o proveedor especifico. Cada fuente de datos debe conectarse a
  traves de una capa comun.
- Historial auditable: cada transaccion debe conservar su origen, cambios
  relevantes y estado de conciliacion.
- Conciliacion explicita: el sistema debe ayudar a detectar diferencias entre el
  saldo real y el saldo calculado, sin esconder ajustes ni borrar historial.
- Proyecciones explicables: los escenarios y forecast deben mostrar de donde sale
  cada supuesto.

## Actores

- Usuario personal: persona que quiere entender ingresos, gastos, deudas,
  inversiones y flujo mensual.
- Usuario compartido: persona que comparte gastos con pareja, familia,
  roomies, amigos o grupos.
- Fuente financiera: banco, tarjeta, fintech, plataforma de inversion, Splitwise,
  archivo importado o API basada en Open Finance.
- Motor de proyeccion: componente que calcula flujo futuro con historia y
  estimaciones del usuario.
- Herramienta externa: otro sistema, script, agente o asistente que consulta o
  modifica datos mediante una API publica o MCP.

## Capa de integraciones financieras

La aplicacion debe prepararse para conectarse a distintas fuentes sin acoplar el
dominio principal a una implementacion particular.

### Objetivo

Definir un contrato interno estable para cuentas, balances, transacciones y
metadatos, de modo que se puedan conectar fuentes actuales y futuras:

- Importacion manual por archivo.
- APIs publicas de instituciones como Fintual.
- Integraciones con plataformas bancarias chilenas mientras no exista una mejor
  opcion.
- Proveedores tipo Fintoc u otros agregadores.
- APIs basadas en la especificacion chilena de Open Finance cuando las
  instituciones la desplieguen.

### Capacidades esperadas

- Registrar una fuente financiera con tipo, institucion, estado y permisos.
- Sincronizar cuentas y transacciones desde una fuente.
- Normalizar datos externos al modelo interno de Presupuesto.
- Conservar moneda, monto original, monto convertido y tipo de cambio cuando la
  fuente entregue datos multimoneda.
- Detectar duplicados entre fuentes y transacciones manuales.
- Exponer duplicados probables para que el usuario pueda fusionarlos, ignorarlos
  o mantenerlos separados.
- Marcar errores de sincronizacion y permitir reintentos.
- Guardar el identificador externo de cada cuenta o transaccion importada.
- Permitir que una fuente sea reemplazada sin cambiar el historial del usuario.

## Casos de uso

### UC-001: Crear transaccion manual

Como usuario quiero registrar una transaccion manual para mantener mi historial
financiero aunque no exista una integracion disponible.

Flujo principal:

1. El usuario ingresa monto, fecha, descripcion, tipo y cuenta.
2. El usuario puede asignar categoria, etiquetas y notas.
3. El sistema guarda la transaccion como `manual`.
4. La transaccion queda disponible para reportes, objetivos y proyecciones.

Criterios:

- Debe soportar ingresos, gastos y transferencias.
- Debe permitir editar y eliminar la transaccion.
- Debe distinguir transacciones manuales de transacciones importadas.

### UC-002: Editar y categorizar transacciones

Como usuario quiero corregir y clasificar mis transacciones para tener reportes
y proyecciones utiles.

Flujo principal:

1. El usuario revisa una lista de transacciones.
2. El usuario filtra por fecha, cuenta, categoria, origen o texto.
3. El usuario edita categoria, descripcion, etiquetas o notas.
4. El sistema guarda los cambios sin perder el dato original importado.

Criterios:

- Los datos normalizados deben poder diferir del dato original.
- Las categorias deben servir para reportes y presupuestos.
- Los cambios del usuario tienen prioridad sobre sugerencias automaticas.

### UC-003: Importar transacciones automaticamente

Como usuario quiero importar movimientos desde mis cuentas para no registrar todo
a mano.

Flujo principal:

1. El usuario conecta o configura una fuente financiera.
2. El sistema obtiene cuentas disponibles.
3. El usuario elige que cuentas sincronizar.
4. El sistema importa transacciones nuevas.
5. El sistema normaliza y deduplica movimientos.
6. El usuario revisa los movimientos importados.

Criterios:

- Cada transaccion importada debe conservar origen e identificador externo.
- Cada transaccion importada debe conservar moneda y monto original.
- La importacion debe ser incremental cuando la fuente lo permita.
- Los duplicados deben detectarse por fecha, monto, descripcion, cuenta y origen.
- Debe existir un estado de revision para movimientos nuevos o ambiguos.

### UC-004: Conectar con Open Finance Chile

Como usuario quiero conectar instituciones financieras chilenas via Open Finance
cuando este disponible para importar informacion de forma estandar y consentida.

Flujo principal esperado:

1. El usuario elige una institucion compatible.
2. El sistema inicia el flujo de consentimiento/autorizacion.
3. La institucion o proveedor devuelve permisos y tokens de acceso.
4. El sistema sincroniza cuentas, balances y transacciones.
5. El usuario puede revocar o pausar la conexion.

Criterios:

- La implementacion debe usar un adaptador especifico de Open Finance.
- El dominio interno no debe depender de campos especificos del proveedor.
- Debe quedar registro de permisos, expiracion y estado de consentimiento.
- Debe ser posible simular esta integracion antes de que los bancos la desplieguen.

### UC-005: Conectar con Fintual u otras APIs publicas

Como usuario quiero integrar plataformas financieras con API publica para incluir
inversiones o productos no bancarios en mi vision financiera.

Flujo principal:

1. El usuario registra una credencial o token para la institucion.
2. El sistema obtiene productos, balances o movimientos disponibles.
3. El sistema normaliza esa informacion como cuentas, posiciones o transacciones.
4. El usuario decide si esos datos participan en reportes y proyecciones.

Criterios:

- La integracion debe vivir en un adaptador propio.
- Debe distinguir cuentas transaccionales de inversiones u otros productos.
- Debe poder desactivarse sin borrar el historial normalizado.

### UC-006: Conectar con Splitwise

Como usuario quiero unir gastos compartidos de Splitwise con mis transacciones
para entender cuanto pague realmente y cuanto me deben o debo.

Flujo principal:

1. El usuario conecta Splitwise.
2. El sistema importa grupos, gastos y saldos.
3. El usuario vincula un gasto de Splitwise con una transaccion bancaria o manual.
4. El sistema calcula la parte efectiva del usuario.
5. El sistema registra saldos pendientes con otras personas.

Criterios:

- Un gasto compartido puede estar asociado a una transaccion real.
- Debe poder reflejar pagos, deudas y reembolsos.
- Debe evitar contar dos veces el mismo gasto.

### UC-007: Ver presupuesto y flujo mensual

Como usuario quiero ver mi situacion mensual para saber cuanto ingreso, gasto,
ahorro y deuda tengo.

Flujo principal:

1. El usuario abre una vista de periodo.
2. El sistema muestra ingresos, gastos, balance neto y categorias principales.
3. El sistema separa movimientos confirmados, pendientes y estimados.
4. El usuario puede entrar al detalle de cada categoria o cuenta.

Criterios:

- Debe permitir comparar meses.
- Debe separar gastos recurrentes de gastos variables.
- Debe permitir ver montos brutos y montos ajustados por gastos compartidos.

### UC-008: Crear proyeccion basada en historia

Como usuario quiero proyectar mis proximos meses usando mi comportamiento
historico para anticipar problemas de caja.

Flujo principal:

1. El usuario elige un horizonte de tiempo.
2. El sistema calcula ingresos y gastos recurrentes desde la historia.
3. El sistema estima gastos variables por categoria.
4. El sistema muestra flujo futuro y saldo esperado.

Criterios:

- La proyeccion debe mostrar supuestos usados.
- El usuario debe poder excluir categorias o transacciones atipicas.
- Debe distinguir datos historicos de estimaciones.

### UC-009: Crear proyeccion con estimados del usuario

Como usuario quiero ajustar una proyeccion con valores esperados para modelar mi
realidad mejor que un promedio historico.

Flujo principal:

1. El usuario define ingresos, gastos o eventos futuros esperados.
2. El sistema combina historia y estimaciones manuales.
3. El usuario revisa el impacto en flujo, ahorro y objetivos.

Criterios:

- Los estimados deben tener fecha, monto, categoria y recurrencia opcional.
- Los estimados no deben confundirse con transacciones reales.
- Debe ser posible convertir un estimado en transaccion cuando ocurra.

### UC-010: Manejar cuentas y transacciones multimoneda

Como usuario quiero ver cuentas y movimientos en su moneda real y tambien su
equivalente estimado o final en CLP para entender mi posicion total.

Flujo principal:

1. El usuario registra o importa una cuenta con moneda propia, por ejemplo CLP,
   USD o EUR.
2. Si la cuenta no define moneda propia, hereda la moneda global del usuario o
   workspace.
3. El sistema muestra el saldo en la moneda original.
4. El sistema calcula un equivalente en la moneda principal del usuario.
5. El usuario revisa transacciones con monto original, moneda y conversion.
6. Cuando una transaccion se liquida, el sistema guarda el tipo de cambio real
   usado para el pago si esta disponible.

Ejemplos:

- Cuenta corriente CLP: saldo `1.000.000 CLP`.
- Cuenta Wise USD: saldo `2.000 USD`, equivalente estimado `1.900.000 CLP`.
- Compra internacional: monto original `100 USD`, conversion estimada mientras
  esta pendiente y conversion final cuando se factura o paga.

Criterios:

- Toda cuenta debe tener una moneda base.
- Debe existir una moneda global configurable para reportes consolidados.
- Las cuentas deben heredar la moneda global salvo que el usuario la sobrescriba.
- Toda transaccion debe conservar monto y moneda original.
- Los reportes consolidados deben usar una moneda principal configurable,
  inicialmente CLP.
- Debe distinguir conversion estimada de conversion final.
- Debe guardar la fuente del tipo de cambio: proveedor, institucion, manual o
  estimacion del sistema.
- Debe permitir que el usuario corrija manualmente el tipo de cambio si la
  institucion cobra uno distinto.
- Las proyecciones deben poder usar tipos de cambio estimados para movimientos
  futuros o pendientes.
- El sistema debe evitar mezclar montos de monedas distintas sin conversion
  explicita.

### UC-011: Configurar periodo de presupuesto

Como usuario quiero definir cuando empieza y termina mi presupuesto mensual para
que los reportes calcen con mi sueldo, tarjetas y forma real de pagar.

Flujo principal:

1. El usuario elige una regla de inicio de periodo.
2. El sistema calcula los periodos de presupuesto desde esa regla.
3. El usuario revisa transacciones agrupadas por periodo.
4. El usuario ajusta reglas especiales para cuentas o tarjetas si lo necesita.

Ejemplos de reglas:

- El dia `1` de cada mes.
- El dia `25` de cada mes.
- El ultimo viernes de cada mes.
- Una fecha manual para un periodo especifico.

Criterios:

- Los reportes mensuales deben usar periodos de presupuesto, no solo meses
  calendario.
- La regla debe soportar dias fijos y reglas relativas como ultimo viernes.
- Debe permitir excepciones por cuenta, tarjeta o periodo.
- El sistema debe mostrar claramente a que periodo pertenece cada transaccion.

### UC-012: Mover transacciones a otro periodo

Como usuario quiero mover ciertas transacciones al siguiente periodo para que el
presupuesto refleje cuando realmente las voy a pagar.

Flujo principal:

1. El usuario define una regla de rollover para una cuenta o tarjeta.
2. El sistema identifica transacciones que caen entre el corte de la cuenta y el
   inicio del presupuesto.
3. El sistema asigna esas transacciones al periodo siguiente.
4. El usuario puede revisar y corregir la asignacion.

Ejemplo:

- La tarjeta factura el dia `24`.
- El presupuesto reinicia el dia `1`.
- Las compras hechas entre el `25` y el `1` se consideran parte del siguiente
  periodo, porque se pagaran en la siguiente facturacion.

Criterios:

- El rollover debe cambiar el periodo presupuestario, no la fecha real de la
  transaccion.
- Debe conservar la fecha contable/original y la fecha o periodo presupuestario.
- Debe poder aplicarse por cuenta, tarjeta, categoria o transaccion individual.
- Debe evitar duplicar gastos al moverlos entre periodos.

### UC-013: Definir objetivos financieros

Como usuario quiero crear objetivos para seguir ahorro, pago de deudas o compras
importantes.

Flujo principal:

1. El usuario crea un objetivo con nombre, monto y fecha esperada.
2. El sistema calcula avance y ritmo necesario.
3. El usuario asocia cuentas, categorias o aportes al objetivo.
4. El sistema muestra si el objetivo va adelantado o atrasado.

Criterios:

- Debe soportar objetivos de ahorro y reduccion de deuda.
- Debe soportar objetivos de compra o gasto futuro, como comprar un monitor,
  viajar o pagar una cuenta anual.
- Debe poder incluir aportes manuales o detectados desde transacciones.
- Debe alimentar las proyecciones.
- Debe distinguir entre dinero disponible, dinero reservado para objetivos y
  dinero reservado para emergencia.
- Debe permitir asociar un objetivo a cuentas o sobres que pueden financiarlo.

### UC-014: Planificar escenarios

Como usuario quiero comparar distintos planes sobre una misma proyeccion para
tomar decisiones financieras antes de ejecutarlas.

Flujo principal:

1. El usuario crea un escenario desde una proyeccion base.
2. El usuario define reglas de asignacion para el dinero disponible.
3. El usuario agrega, quita o modifica objetivos del escenario.
4. El sistema calcula cuanto queda libre, reservado para objetivos y reservado
   para emergencia.
5. El sistema muestra diferencias contra el escenario base.
6. El usuario guarda o descarta el escenario.

Ejemplos de planes:

- Plan base: vivir sin metas adicionales. El dinero que entra queda disponible
  para uso general, descontando gastos reales y estimados.
- Plan de objetivos: reservar dinero para metas o gastos futuros, como comprar
  un monitor dentro del año. El sistema muestra cuanto se necesita, cuanto ya
  esta reservado y cuanto falta.
- Plan de emergencia: reservar dinero en cuentas o sobres especificos que no se
  consideran disponibles para otros usos.

Ejemplo de lectura:

- Dinero total actual: `5.000.000`.
- Objetivos: meta `1.000.000`, avance `400.000 / 1.000.000`.
- Emergencia: meta `3.000.000`, avance `140.000 / 3.000.000`.

Reglas de reserva:

- El dinero reservado para emergencia no puede usarse para objetivos comunes.
- El dinero reservado para objetivos puede contarse como respaldo de emergencia
  si el usuario lo permite.
- Esta relacion es direccional: `emergencia efectiva = emergencia + objetivos
  habilitados como respaldo`, pero los objetivos disponibles no deben incluir el
  fondo de emergencia.
- Una cuenta o sobre puede estar marcado como disponible para objetivos,
  exclusivo para emergencia o disponible para uso general.
- El sistema debe evitar contar el mismo peso dos veces cuando una cuenta sirve
  como respaldo para mas de una lectura.

Criterios:

- Un escenario no debe modificar transacciones reales.
- Debe poder comparar al menos dos escenarios.
- Debe mostrar impacto en flujo mensual, saldo final y objetivos.
- Debe mostrar brechas por plan: cuanto tengo, cuanto quiero reservar y cuanto
  falta.
- Debe permitir que cada escenario tenga reglas distintas sobre que cuentas o
  sobres financian objetivos y emergencia.
- Debe explicar las reglas de prioridad y fallback usadas en el calculo.

### UC-015: Proyectar escenario negativo o de supervivencia

Como usuario quiero simular una perdida de ingresos u otro evento negativo para
saber cuanto tiempo puedo cubrir gastos basicos y que cuentas se agotarian
primero.

Flujo principal:

1. El usuario crea un escenario negativo desde una proyeccion base.
2. El usuario reduce o elimina ingresos esperados, por ejemplo por despido.
3. El usuario define gasto minimo mensual o selecciona categorias esenciales,
   como arriendo, comida, servicios basicos, salud y transporte.
4. El usuario define el orden de uso de cuentas o sobres.
5. El sistema calcula cuanto dura cada cuenta y cuando se agota.
6. El sistema muestra la duracion total del plan de supervivencia.

Ejemplo de lectura:

- Cuentas:
  - Cuenta corriente: `1.000.000`.
  - Fondos: `2.000.000`.
  - Cuenta de ahorro: `4.000.000`.
  - Fondo de retiro: `10.000.000`.
- Gasto minimo mensual: `500.000`.
- Resultado:
  - Cuenta corriente dura `2` meses.
  - Fondos duran `4` meses adicionales, hasta el mes `6`.
  - Cuenta de ahorro dura `8` meses adicionales, hasta el mes `14`.
  - Fondo de retiro se empieza a usar despues del mes `14`, si el usuario lo
    permite.

Criterios:

- Debe permitir definir cuentas que se usan primero, despues o nunca.
- Debe poder excluir cuentas bloqueadas, de retiro o de emergencia extrema.
- Debe mostrar hitos de agotamiento por cuenta y fecha/mes esperado.
- Debe convertir saldos multimoneda usando tipos de cambio estimados y mostrar
  el riesgo de variacion cambiaria cuando corresponda.
- Debe mover automaticamente dinero reservado para objetivos hacia gastos
  esenciales si el escenario lo permite.
- Debe respetar reservas que el usuario marque como intocables.
- Debe distinguir gastos esenciales de gastos discrecionales.
- Debe permitir ajustar el gasto mensual minimo manualmente.

### UC-016: Revisar y resolver transacciones duplicadas

Como usuario quiero revisar duplicados probables para evitar que una misma
compra, transferencia o gasto compartido se cuente dos veces.

Flujo principal:

1. El sistema detecta transacciones similares por fecha, monto, moneda, cuenta,
   descripcion, contraparte, origen o identificador externo.
2. El sistema muestra un grupo de duplicados probables.
3. El usuario decide fusionar, descartar una copia, mantenerlas separadas o
   ignorar la sugerencia.
4. El sistema conserva el historial de origen y la decision del usuario.

Criterios:

- La deteccion automatica no debe borrar transacciones sin confirmacion.
- Debe distinguir duplicados reales de pagos repetidos validos.
- Debe soportar duplicados entre transacciones manuales, importadas, Splitwise y
  archivos.
- Debe registrar que fuentes externas quedaron asociadas a la transaccion final.
- Debe permitir deshacer una fusion cuando sea necesario.

### UC-017: Conciliar cuentas

Como usuario quiero conciliar una cuenta cuando el saldo calculado por el sistema
no coincide con el saldo real del banco, efectivo, billetera o plataforma.

Flujo principal:

1. El usuario abre una cuenta y compara saldo calculado contra saldo real.
2. El usuario ingresa el saldo real y la fecha de conciliacion.
3. El sistema calcula la diferencia.
4. El usuario revisa si faltan transacciones, sobran duplicados o corresponde un
   ajuste.
5. El sistema registra la conciliacion y, si el usuario lo confirma, crea una
   transaccion de ajuste.

Ejemplo:

- El sistema calcula `95.000 CLP` en efectivo.
- El usuario cuenta `80.000 CLP`.
- La diferencia es `-15.000 CLP`, posiblemente por una compra en efectivo no
  registrada.

Criterios:

- La conciliacion debe conservar el saldo anterior, saldo real, diferencia,
  fecha y nota.
- El ajuste debe ser visible como ajuste de conciliacion, no como gasto normal
  sin explicacion.
- Debe permitir asociar la diferencia a una categoria si el usuario sabe que
  corresponde a un gasto real.
- Debe funcionar para cuentas manuales, bancarias, efectivo y multimoneda.
- Debe ayudar a encontrar transacciones faltantes o duplicadas antes de crear un
  ajuste.

### UC-018: Preparar informacion para declaracion anual en Chile

Como usuario quiero consolidar informacion de ingresos, inversiones y otros
datos tributarios para preparar reportes que ayuden con la declaracion anual en
Chile.

Flujo principal:

1. El usuario importa o registra datos tributarios entregados por empresas,
   corredoras, fintechs u otras instituciones.
2. El sistema normaliza conceptos, periodos, monedas y montos.
3. El sistema agrupa datos equivalentes entre instituciones.
4. El sistema genera un resumen anual exportable.

Criterios:

- Este modulo puede vivir dentro del producto o como proyecto separado que lea
  datos desde Presupuesto.
- Debe conservar fuente, documento, fecha, moneda y monto original.
- Debe soportar datos de inversiones, dividendos, ventas, intereses u otros
  conceptos a definir.
- Debe exportar informacion en formatos revisables por el usuario.
- No debe prometer declaracion automatica ni reemplazar revision tributaria.

### UC-019: Exponer API publica o MCP

Como usuario avanzado o herramienta externa quiero interactuar con Presupuesto
mediante una API o MCP para consultar datos, automatizar flujos y construir
integraciones.

Flujo principal:

1. El usuario habilita acceso externo.
2. El sistema crea credenciales o permisos acotados.
3. La herramienta externa consulta cuentas, transacciones, reportes o
   proyecciones.
4. La herramienta externa crea o modifica datos permitidos.
5. El usuario puede auditar y revocar el acceso.

Criterios:

- Esta capacidad no es prioritaria para el inicio; deberia implementarse hacia
  la mitad o final del desarrollo.
- Debe tener permisos por alcance: lectura, escritura, reportes,
  conciliacion, integraciones.
- Debe registrar acciones hechas por herramientas externas.
- Debe evitar exponer credenciales bancarias o tokens de integraciones.
- Debe permitir que otros proyectos lean datos para reportes tributarios,
  automatizaciones o asistentes personales.

## Reportes

Los reportes deben responder preguntas concretas y usar la moneda global del
usuario o workspace para vistas consolidadas. Cuando incluyan cuentas o
transacciones multimoneda, deben mostrar la moneda original y la conversion usada.

### Reportes prioritarios para el PoC

- Salud financiera: muestra si el usuario esta ahorrando suficiente, si la deuda
  esta controlada, si hay fondo de emergencia y si los gastos son sostenibles.
  Debe partir con reglas predefinidas y permitir reglas configurables mas
  adelante.
- Periodo en curso contra proyeccion: compara gasto, ingreso, ahorro y reservas
  esperadas contra lo real mientras el periodo sigue abierto.
- Cierre de periodo contra proyeccion: muestra como termino el periodo contra lo
  proyectado, que categorias explican la diferencia y que supuestos fallaron.
- Desglose de gasto: muestra en que se esta gastando dinero por categoria,
  cuenta, comercio, etiqueta, periodo y tipo de gasto.
- Flujo de caja: resume ingresos, gastos fijos, gastos variables, ahorro, pagos
  de deuda y resultado neto por periodo.
- Objetivos y reservas: muestra avance de metas, fondo de emergencia, dinero
  reservado, dinero disponible y conflictos entre reservas.
- Runway o supervivencia: muestra cuantos meses de gastos esenciales estan
  cubiertos, que cuentas financian ese runway y cuando se agota cada una.

### Reportes adicionales

- Deriva presupuestaria: detecta categorias o cuentas que se estan alejando de
  la proyeccion antes de que cierre el periodo.
- Gastos recurrentes: lista suscripciones, cuentas, deudas, ingresos esperados y
  transacciones repetitivas que alimentan la proyeccion.
- Deuda: muestra tarjetas, creditos, cuotas, deuda total, obligacion mensual,
  intereses si existen y progreso de pago.
- Asignacion de cuentas: compara donde esta fisicamente el dinero contra su
  proposito, por ejemplo cuenta corriente, ahorro, USD, inversiones, emergencia
  y objetivos.
- Patrimonio neto: muestra activos menos deudas en el tiempo, incluyendo
  inversiones y saldos multimoneda.
- Anomalias: identifica gastos inusuales, cargos inesperados, duplicados
  probables o cambios relevantes frente a la historia.
- Gastos compartidos: muestra gastos de Splitwise u otros grupos, dinero por
  cobrar, dinero por pagar e impacto en gasto real.
- Exposicion cambiaria: muestra saldos y transacciones pendientes por moneda,
  valor estimado en moneda global, conversion final y riesgo cambiario.
- Precision de proyecciones: compara proyectado contra real por varios periodos,
  categoria y cuenta para mejorar supuestos futuros.
- Resumen anual o tributario: consolida informacion anual util para revision
  personal, inversiones o preparacion de declaracion de impuestos.

## Modelo conceptual inicial

- Cuenta: producto financiero o contenedor manual de dinero.
- Transaccion: movimiento de dinero real o registrado manualmente.
- Moneda: codigo de moneda de una cuenta, transaccion, estimado o reporte.
- Moneda global: moneda principal usada para reportes consolidados de un usuario
  o workspace.
- Tipo de cambio: valor usado para convertir un monto entre monedas.
- Conversion: monto convertido, moneda destino, tipo de cambio, fuente y estado.
- Fuente financiera: origen externo o manual de datos.
- Categoria: clasificacion principal para reportes.
- Etiqueta: clasificacion flexible adicional.
- Gasto compartido: gasto, deuda o reembolso asociado a otra persona o grupo.
- Estimado: movimiento futuro esperado que aun no es transaccion real.
- Objetivo: meta financiera con monto, plazo y avance.
- Plan: politica de asignacion de dinero sobre una proyeccion.
- Escenario: combinacion de una proyeccion, sus estimados y uno o mas planes.
- Sobre: reserva logica de dinero que puede estar asociada a una o mas cuentas.
- Reserva: monto separado para un objetivo, emergencia u otro proposito.
- Regla de fallback: permiso para que una reserva cuente como respaldo de otra
  lectura sin cambiar su proposito principal.
- Periodo de presupuesto: rango de fechas usado para reportes y control mensual.
- Regla de periodo: definicion de inicio/cierre del presupuesto, como dia fijo o
  regla relativa.
- Regla de rollover: criterio que asigna transacciones a un periodo distinto al
  de su fecha real.
- Orden de liquidacion: prioridad para usar cuentas o sobres en un escenario
  negativo.
- Conciliacion: comparacion entre saldo calculado y saldo real de una cuenta en
  una fecha determinada.
- Ajuste de conciliacion: transaccion explicita que corrige una diferencia de
  saldo cuando no se puede o no se quiere reconstruir el detalle.
- Reporte: vista calculada que resume estado financiero, periodo, proyeccion o
  historial.
- Acceso externo: permiso entregado a una API, MCP, script o herramienta para
  interactuar con datos de Presupuesto.

## Preguntas abiertas

- Backend: Supabase, otro BaaS o backend propio.
- Alcance del primer PoC: solo transacciones manuales o importacion desde el
  inicio.
- Profundidad de la integracion con Splitwise: solo importacion o tambien
  conciliacion bidireccional.
- Alcance de inversiones: balances simples o movimientos/posiciones detalladas.
- Estrategia de seguridad para credenciales y tokens de integraciones.
- Modelo de permisos y consentimiento para Open Finance.
- Como modelar cuentas versus sobres: una cuenta real puede financiar varios
  sobres, o un sobre puede agregarse sobre varias cuentas.
- Como priorizar reservas cuando el flujo proyectado no alcanza para todos los
  objetivos.
- Si el periodo de presupuesto debe ser global por usuario o configurable por
  workspace/hogar.
- Como representar tarjetas de credito: por fecha de compra, fecha de facturacion
  o fecha de pago.
- Que restricciones aplicar a cuentas de retiro o inversion cuando se usan en
  escenarios negativos.
- Que fuente usar para tipos de cambio estimados y con que frecuencia
  actualizarlos.
- Como tratar comisiones, spreads y cargos internacionales cuando difieren del
  tipo de cambio publicado.
- Que reglas predefinidas de salud financiera usar para Chile y cuales deben ser
  configurables por el usuario.
- Como representar conciliaciones recurrentes sin crear ajustes innecesarios.
- Que datos tributarios chilenos conviene modelar dentro del producto y cuales
  dejar para un proyecto externo.
- Que superficie minima deberia exponer una API o MCP sin comprometer seguridad
  ni privacidad.
