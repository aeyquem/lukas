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
- Las transacciones deben soportar categoria principal y subcategoria opcional.
- La categoria principal debe servir para reportes generales y la subcategoria
  para analisis mas especifico.
- Una transaccion puede tener categoria sin subcategoria.
- Transferencias, asignaciones, prestamos/deudas entre personas y ajustes de
  conciliacion deben poder distinguirse de categorias normales de gasto.
- Las transacciones deben soportar multiples etiquetas personalizadas
  independientes de categoria y subcategoria.
- Las etiquetas deben servir para contexto transversal como viajes, proyectos,
  personas, revision tributaria, reembolsable o gasto laboral.
- Las etiquetas no deben estar limitadas por el arbol de categorias.
- Las etiquetas importadas desde una fuente deben conservarse como metadata, pero
  las etiquetas definidas por el usuario tienen prioridad en la interfaz.
- Las categorias, subcategorias y etiquetas editadas por el usuario deben
  conservarse aunque la transaccion se importe nuevamente.
- La clasificacion importada desde la fuente debe guardarse como metadata de
  origen separada de la clasificacion del usuario.
- Futuras sincronizaciones no deben sobrescribir clasificaciones del usuario sin
  confirmacion explicita.
- Si la fuente cambia su clasificacion, el sistema puede mostrarlo como
  sugerencia o cambio de metadata.
- Las reglas masivas definidas por el usuario pueden actualizar categorias,
  subcategorias o etiquetas de forma intencional.
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
- Una misma transaccion real puede aparecer desde banco, tarjeta, CSV, Open
  Finance, Splitwise o registro manual.
- El sistema debe poder vincular multiples registros de origen a una transaccion
  logica sin perder la trazabilidad de cada fuente.
- La vinculacion debe evitar doble conteo en reportes.
- El identificador interno de Presupuesto debe ser estable y no depender del
  identificador externo del proveedor.
- Los identificadores externos deben conservarse como metadata de origen y
  pueden cambiar, faltar o repetirse segun la fuente.
- Si una fuente cambia un identificador externo, el sistema debe usar otras
  senales de coincidencia antes de crear una transaccion nueva.
- Si un identificador externo parece reutilizado o ambiguo, el movimiento debe
  quedar en revision del usuario.
- Si una institucion cambia una transaccion historica, el sistema debe detectar
  la mutacion y conservar que campos cambiaron.
- Los cambios de pendiente a final pueden actualizar la transaccion normalmente
  mientras conserven historial de origen.
- Si la transaccion ya fue revisada, conciliada o pertenece a un periodo cerrado,
  el usuario debe decidir si acepta el cambio, lo ignora, conserva ambos valores
  o crea un ajuste.
- Los reportes deben conservar trazabilidad entre el valor importado original, la
  mutacion recibida y la decision del usuario.
- Si el usuario oculta, descarta o elimina un registro importado, el sistema debe
  conservar una marca de descarte o tombstone asociada a la fuente.
- La marca debe guardar fuente, identificador externo si existe, firma de
  coincidencia, razon, fecha y decision del usuario.
- Las sincronizaciones futuras deben respetar esas marcas para no recrear
  movimientos descartados.
- Si la fuente vuelve a enviar datos materialmente distintos, el sistema puede
  llevar el caso nuevamente a revision.
- Las marcas de descarte no deben borrar historial de auditoria ni vinculos con
  transacciones existentes.
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
- Un gasto de Splitwise vinculado a una transaccion bancaria o manual debe
  explicar la parte compartida, deuda o reembolso sin duplicar el movimiento real
  de dinero.

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
- Broker chileno: balance reportado `5.000.000 CLP`, aunque los activos
  subyacentes puedan estar valorizados en otra moneda dentro de la institucion.

Criterios:

- Toda cuenta debe tener una moneda base.
- Debe existir una moneda global configurable para reportes consolidados.
- Las cuentas deben heredar la moneda global salvo que el usuario la sobrescriba.
- Toda transaccion debe conservar monto y moneda original.
- Los reportes consolidados deben usar una moneda principal configurable,
  inicialmente CLP.
- Las cuentas de inversion deben usar como valor principal el balance y moneda
  reportados por la institucion.
- El sistema no necesita modelar monedas, precios o posiciones subyacentes de
  activos cuando el usuario puede revisar ese detalle en la institucion.
- Una transaccion multimoneda puede conservar varios valores monetarios:
  monto del comercio, monto autorizado, monto facturado, monto pagado,
  comisiones, impuestos y cargo final, cuando la fuente los entregue.
- Debe distinguir conversion estimada de conversion final.
- Debe guardar la fuente del tipo de cambio: institucion, red de pago, broker,
  billetera, proveedor, manual o estimacion del sistema.
- Cuando existan varias conversiones para una transaccion, cada conversion debe
  conservar su propia fuente, fecha, estado y valor.
- El sistema debe permitir explicar al usuario por que dos operaciones similares
  usaron tipos de cambio distintos.
- Una misma moneda y fecha puede tener distintos tipos de cambio segun banco,
  tarjeta, red de pago, broker, proveedor o correccion manual.
- Los reportes deben usar la conversion guardada en la transaccion cuando
  exista, en vez de asumir un tipo de cambio diario unico.
- Debe permitir que el usuario corrija manualmente el tipo de cambio si la
  institucion cobra uno distinto.
- La vista principal debe mostrar el monto original y el impacto actual o final
  en la cuenta, dejando los valores intermedios como detalle auditable.
- El sistema no debe inventar comisiones o impuestos separados cuando la fuente
  solo entrega un cargo final.
- Las proyecciones deben poder usar tipos de cambio estimados para movimientos
  futuros o pendientes.
- Para transacciones internacionales pendientes, el sistema debe preferir el
  valor convertido pendiente entregado por la institucion antes que una
  estimacion propia.
- Si una tarjeta esta configurada para estimar con tipo de cambio de red o
  emisor, como Visa o Mastercard, el sistema puede usar esa fuente y sumar una
  comision configurada, por ejemplo un porcentaje de la transaccion.
- La regla de estimacion por tarjeta es opcional; si no existe, el sistema debe
  usar la regla general de estimacion fallback.
- Si la institucion no entrega valor convertido ni tipo de cambio pendiente, el
  sistema puede usar una estimacion fallback marcada como estimada.
- Cuando exista valor final facturado o pagado, los reportes deben usar ese
  valor final y conservar el valor pendiente solo como detalle historico.
- Los reembolsos internacionales deben registrarse como transacciones separadas
  asociadas a la compra original, no como una reescritura del cargo original.
- Un reembolso puede usar un tipo de cambio distinto al de la compra original;
  los reportes deben calcular el efecto neto usando los valores finales reales
  de ambas transacciones.
- El sistema debe soportar reembolsos parciales sin asumir que cancelan la compra
  completa ni que usan el mismo tipo de cambio.
- Los reportes historicos deben usar la conversion guardada en cada transaccion
  cuando exista.
- La busqueda historica de tipo de cambio solo debe usarse cuando falte una
  conversion guardada.
- Cuando el sistema genere una conversion historica fallback, debe guardarla con
  fuente y fecha para que los reportes futuros sean repetibles.
- El sistema no debe recalcular transacciones historicas liquidadas solo porque
  un proveedor actualizo datos pasados de tipo de cambio.
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
- El inicio del periodo de presupuesto debe usar la fecha configurada tal cual,
  incluso si cae en fin de semana o feriado, salvo ajuste manual explicito.
- Debe permitir excepciones por cuenta, tarjeta o periodo.
- El sistema debe mostrar claramente a que periodo pertenece cada transaccion.
- Un periodo puede estar abierto, revisado, bloqueado o reabierto.
- Los periodos bloqueados deben mantener estables sus totales historicos.
- Cambios que afecten un periodo bloqueado deben requerir una decision explicita:
  reabrir, crear ajuste posterior al cierre, aplicar en un periodo abierto o
  ignorar.
- Los ajustes posteriores al cierre deben quedar visibles en reportes y
  auditoria.
- Reabrir un periodo debe ser una accion intencional y registrada.

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

- Las transacciones de tarjeta pueden conservar fechas separadas de compra,
  autorizacion, facturacion, vencimiento y pago.
- Los reportes de gasto deben usar la fecha de compra por defecto, mientras que
  la deuda, facturacion y flujo de caja pueden usar fechas de facturacion,
  vencimiento o pago segun el reporte.
- Si la fuente solo entrega una fecha, el sistema debe conservarla como mejor
  fecha disponible y usar reglas de la tarjeta para inferir facturacion o pago
  cuando sea posible.
- Las reglas de facturacion de una tarjeta pueden considerar desfases por fin de
  semana o feriado cuando la cartola se emite el dia habil siguiente.
- Una compra hecha en fin de semana puede pertenecer a la cartola emitida el
  lunes siguiente si la institucion la incluye en ese cierre.
- Estos desfases afectan la asignacion a cartola o facturacion, no cambian por
  si solos el inicio del periodo de presupuesto.
- Las compras en cuotas deben poder representarse como una compra original que
  genera cargos futuros en la tarjeta.
- La compra en cuotas debe conservar monto total, cantidad de cuotas, primer
  periodo de facturacion y calendario de cargos cuando se conozca.
- Si la fuente importa cada cuota como cargo separado, el sistema debe poder
  vincularlas a la compra original sin duplicar el gasto.
- La v1 no necesita calcular tasa de interes implicita; intereses o comisiones
  solo se registran por separado si la fuente los entrega o el usuario los
  ingresa.
- Intereses, mora, mantencion, seguros, impuestos y cargos de servicio de
  tarjeta deben registrarse como costos visibles, no como metadata oculta.
- Si la fuente no permite distinguir el detalle del cargo, el sistema puede usar
  una categoria por defecto de costo de mantencion o uso de tarjeta.
- El usuario debe poder recategorizar esos costos cuando tenga mas informacion.
- Los costos de tarjeta deben afectar reportes de gasto y flujo, pero mantenerse
  distinguibles de compras a comercios.
- Los pagos de tarjeta deben registrarse como pagos o transferencias contra la
  cuenta de tarjeta, no como gastos normales.
- Si el usuario paga menos que el saldo facturado, el sistema debe mantener la
  diferencia como deuda pendiente de la tarjeta.
- Los reportes deben poder mostrar saldo facturado, pagos realizados, deuda
  restante y fecha de vencimiento cuando existan esos datos.
- La v1 no necesita estimar intereses rotativos; si la institucion importa
  intereses, deben registrarse como cargos visibles.
- El flujo de caja debe usar el monto efectivamente pagado o planificado, aunque
  sea menor que el saldo facturado.
- Una tarjeta puede quedar con saldo a favor por sobrepago, devolucion o ajuste
  de la institucion.
- El saldo a favor debe tratarse como balance valido de la tarjeta, no como
  ingreso ni como error de conciliacion.
- Compras posteriores pueden consumir el saldo a favor antes de aumentar la deuda
  de la tarjeta.
- Los reportes deben distinguir sobrepago o saldo a favor de gasto, ingreso y
  deuda pendiente.
- Si una cartola o institucion corrige un cargo de tarjeta despues de que el
  periodo fue revisado o cerrado, el sistema no debe reescribir silenciosamente
  el historial.
- El usuario debe poder aceptar la correccion, ignorarla o mantener ambos valores
  cuando representen movimientos distintos.
- Si el periodo esta bloqueado, una correccion aceptada debe registrarse como
  ajuste posterior al cierre o en un periodo abierto segun la regla configurada.
- Los reportes deben conservar trazabilidad entre el valor original visto por el
  usuario, la correccion recibida y la decision tomada.
- Ingresos recurrentes como sueldo, pension o pagos de clientes pueden tener una
  regla de asignacion al periodo esperado aunque lleguen antes por fin de semana
  o feriado.
- La fecha real de la transaccion no debe cambiar; solo cambia el periodo de
  presupuesto asignado.
- Estas reglas pueden configurarse por pagador, cuenta, descripcion recurrente o
  fuente.
- Los reportes deben mostrar cuando un ingreso fue asignado a un periodo distinto
  al de su fecha real.
- Una transaccion puede pertenecer a distintos contextos de periodo al mismo
  tiempo: calendario, presupuesto, cartola, flujo de caja o tributario.
- La fecha real de la transaccion debe guardarse separada de sus asignaciones de
  periodo.
- Cada reporte debe declarar o usar claramente que contexto de periodo esta
  mostrando.
- Un ajuste manual de periodo debe afectar solo el contexto elegido, por ejemplo
  presupuesto, sin reescribir cartola o fecha calendario.
- El rollover debe cambiar el periodo presupuestario, no la fecha real de la
  transaccion.
- Debe conservar la fecha contable/original y la fecha o periodo presupuestario.
- Debe poder aplicarse por cuenta, tarjeta, categoria o transaccion individual.
- Debe evitar duplicar gastos al moverlos entre periodos.

### UC-013: Registrar transferencias entre cuentas propias

Como usuario quiero registrar movimientos entre mis propias cuentas para
mantener saldos correctos sin que esos movimientos se cuenten como ingreso o
gasto.

Flujo principal:

1. El usuario registra o importa una salida desde una cuenta propia.
2. El usuario registra, importa o vincula la entrada correspondiente en otra
   cuenta propia.
3. El sistema marca ambos movimientos como una transferencia interna.
4. Los saldos de ambas cuentas se actualizan.
5. Los reportes de presupuesto excluyen el efecto neto de la transferencia.

Criterios:

- Una transferencia debe tener cuenta origen y cuenta destino cuando ambas sean
  conocidas.
- La salida y la entrada pueden verse en los libros de cada cuenta.
- Los reportes de presupuesto no deben contar transferencias internas como gasto
  ni ingreso.
- Los reportes por cuenta pueden mostrar transferencias para explicar cambios de
  saldo.
- El sistema debe permitir registrar una transferencia con una sola contraparte
  temporal cuando la otra parte aun no fue importada.
- Una transferencia entre monedas debe conservar monto y moneda de origen, monto
  y moneda de destino, y la conversion usada o implicita.
- Si ambas piernas de la transferencia existen, el sistema puede calcular el tipo
  de cambio implicito desde los montos reales.
- Si la institucion entrega comisiones, spread o impuestos por separado, deben
  registrarse como costos explicitos asociados a la transferencia.
- Si esos costos no vienen separados, el usuario puede ingresarlos manualmente o
  el sistema puede mostrar una comparacion contra una estimacion de referencia.
- El movimiento principal sigue siendo transferencia interna; solo costos
  explicitos deben afectar reportes de gasto financiero.
- El sistema debe sugerir coincidencias entre salidas y entradas importadas que
  parezcan partes de la misma transferencia entre cuentas propias.
- La coincidencia debe considerar monto, fecha, moneda, cuentas, descripcion,
  contraparte e identificadores externos cuando existan.
- Vincular dos piernas importadas no debe borrar los registros originales de cada
  fuente.
- Una transferencia puede quedar pendiente o incompleta si solo una pierna fue
  importada.
- El usuario debe poder confirmar, rechazar, deshacer o descartar una sugerencia
  de transferencia.

### UC-014: Manejar efectivo y retiros

Como usuario quiero tratar retiros de efectivo como movimientos hacia una cuenta
de efectivo para no contar el mismo dinero como gasto antes de usarlo.

Flujo principal:

1. El usuario importa o registra un retiro desde una cuenta bancaria.
2. El sistema lo registra como transferencia hacia una cuenta de efectivo.
3. Si no existe una cuenta de efectivo, el sistema permite crearla.
4. Cuando el usuario gasta efectivo, registra una transaccion separada desde la
   cuenta de efectivo.
5. Los reportes de presupuesto cuentan el gasto en efectivo, no el retiro.

Criterios:

- Los retiros de efectivo siempre deben tratarse como transferencias a una cuenta
  de efectivo.
- Si el usuario quiere reflejar el retiro como gasto, debe crear una transaccion
  separada desde la cuenta de efectivo.
- El sistema debe evitar doble conteo entre retiro bancario y gasto registrado en
  efectivo.
- La cuenta de efectivo debe poder ser manual y conciliable.

### UC-015: Definir objetivos financieros

Como usuario quiero crear objetivos para seguir ahorro, pago de deudas o compras
importantes.

Flujo principal:

1. El usuario crea un objetivo con nombre, monto y fecha esperada.
2. El sistema calcula avance y ritmo necesario.
3. El usuario asocia cuentas, categorias o aportes al objetivo.
4. El sistema muestra si el objetivo va adelantado o atrasado.
5. El sistema muestra el fondo total asignado a objetivos y su distribucion
   entre objetivos.

Ejemplo de reasignacion:

- Objetivo A: `30.000 / 50.000`.
- Objetivo B: `40.000 / 70.000`.
- Objetivo C: `25.000 / 100.000`.
- Fondo total asignado a objetivos: `95.000`.
- Si el usuario decide ejecutar B usando fondos de A y B, el resultado puede ser
  A `0 / 50.000`, B `completado`, C `25.000 / 100.000`.

Criterios:

- Debe soportar objetivos de ahorro y reduccion de deuda.
- Debe soportar objetivos de compra o gasto futuro, como comprar un monitor,
  viajar o pagar una cuenta anual.
- Debe poder incluir aportes manuales o detectados desde transacciones.
- Debe alimentar las proyecciones.
- Debe distinguir entre dinero disponible, dinero reservado para objetivos y
  dinero reservado para emergencia.
- Debe existir una lectura de fondo total asignado a objetivos, equivalente a la
  suma del dinero reservado para objetivos.
- Debe permitir asociar un objetivo a cuentas o sobres que pueden financiarlo.
- Cada objetivo debe mostrar cuanto tiene asignado desde el fondo total y cuanto
  falta para poder ejecutarse.
- El usuario debe poder reasignar dinero desde un objetivo a otro sin registrar
  un gasto ni mover dinero entre cuentas reales.
- Antes de ejecutar un objetivo, el sistema debe mostrar que otros objetivos
  quedarian reducidos, pausados, sin fondos o todavia financiados.
- Cada objetivo debe tener monto objetivo y moneda objetivo.
- Un objetivo puede financiarse desde cuentas o sobres en monedas distintas.
- El avance debe calcularse en la moneda del objetivo usando conversiones
  guardadas o estimadas.
- Los reportes deben mostrar monedas originales cuando sean relevantes para
  explicar el avance.
- Si un objetivo depende de fondos en otra moneda, el sistema debe mostrar riesgo
  o variacion cambiaria.
- La moneda objetivo debe heredar la moneda global por defecto, pero puede
  sobrescribirse por objetivo.
- Un objetivo puede estar activo, pausado, completado, cancelado, vencido,
  parcialmente gastado o sobregastado.
- Pausar un objetivo debe detener nuevas contribuciones esperadas sin borrar
  historial ni reservas existentes.
- Cancelar un objetivo debe permitir liberar o reasignar el dinero reservado.
- Si vence la fecha objetivo sin cumplirse, el sistema debe marcarlo como vencido
  o permitir reprogramarlo.
- Gastar parte o mas del monto reservado debe quedar reflejado como consumo
  parcial o sobregasto del objetivo.
- Los cambios de estado deben quedar registrados para auditoria y comparacion de
  escenarios.
- Una compra planificada u objetivo de gasto debe poder vincularse con una o mas
  transacciones reales cuando ocurran.
- Vincular transacciones reales debe consumir o reducir el monto reservado del
  objetivo.
- Un objetivo puede completarse con una sola transaccion, como comprar un auto, o
  con varias transacciones relacionadas, como un viaje.
- El usuario debe poder marcar el objetivo como completado cuando las
  transacciones asociadas representan el cierre del objetivo.
- Si el monto real acumulado difiere del planificado, el sistema debe mostrar la
  variacion.
- Si sobra reserva, el usuario puede mantenerla, liberarla o reasignarla.
- Si falta reserva, el objetivo debe quedar sobregastado o requerir
  financiamiento adicional.
- Los reportes deben evitar contar al mismo tiempo el gasto planificado y las
  transacciones reales como gastos distintos.
- Mover dinero a ahorro, objetivos, emergencia o sobres debe tratarse como
  asignacion interna, no como gasto.
- Las asignaciones pueden afectar dinero disponible, avance de objetivos y
  reservas, pero no deben distorsionar reportes de consumo.
- Los reportes de presupuesto pueden mostrar ahorro y asignaciones separadas de
  ingresos y gastos.
- El gasto ocurre cuando existe una transaccion real hacia una contraparte
  externa; esa transaccion puede consumir una asignacion previa.

### UC-016: Planificar escenarios

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
- Una cuenta puede financiar multiples objetivos o sobres solo mediante montos o
  reglas de asignacion explicitas.
- El sistema no debe contar el mismo dinero como completamente disponible para
  mas de un objetivo, sobre o reserva al mismo tiempo.
- Los reportes deben distinguir saldo real, saldo asignado y saldo disponible sin
  asignar.
- Si las asignaciones superan el saldo disponible, el sistema debe mostrar una
  alerta de sobreasignacion.
- Las reglas de fallback de emergencia pueden contar dinero asignado a objetivos
  como respaldo solo si el usuario lo habilita explicitamente.

Criterios:

- Un escenario no debe modificar transacciones reales.
- Debe poder comparar al menos dos escenarios.
- Debe mostrar impacto en flujo mensual, saldo final y objetivos.
- Debe mostrar brechas por plan: cuanto tengo, cuanto quiero reservar y cuanto
  falta.
- Debe permitir que cada escenario tenga reglas distintas sobre que cuentas o
  sobres financian objetivos y emergencia.
- Debe explicar las reglas de prioridad y fallback usadas en el calculo.

### UC-017: Proyectar escenario negativo o de supervivencia

Como usuario quiero simular una perdida de ingresos u otro evento negativo para
saber cuanto tiempo puedo cubrir gastos basicos y que cuentas se agotarian
primero.

Flujo principal:

1. El usuario crea un escenario negativo desde una proyeccion base.
2. El usuario reduce o elimina ingresos esperados, por ejemplo por despido.
3. Si corresponde, el usuario agrega una compensacion estimada por termino
   laboral o finiquito.
4. El usuario define gasto minimo mensual o selecciona categorias esenciales,
   como arriendo, comida, servicios basicos, salud y transporte.
5. El usuario define el orden de uso de cuentas o sobres.
6. El sistema calcula cuanto dura cada cuenta y cuando se agota.
7. El sistema muestra la duracion total del plan de supervivencia.

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
- El escenario puede definir demora de liquidez por cuenta o producto, por
  ejemplo inmediata, dias habiles o plazo fijo.
- El escenario puede incluir supuestos de penalizaciones, comisiones o impuestos
  por liquidar fondos.
- Estos supuestos deben afectar el calculo de supervivencia, pero no crear
  transacciones reales mientras no ocurran.
- Si la liquidacion, comision o impuesto ocurre realmente, debe registrarse luego
  como transaccion o ajuste real.
- El escenario debe mostrar saldo bruto, monto disponible neto y supuestos usados
  para cada cuenta o producto.
- Para cuentas de inversion, el escenario debe preferir el ultimo balance
  reportado por la institucion.
- Si no existe balance actualizado, el sistema puede usar el ultimo balance
  conocido y marcarlo como desactualizado.
- El usuario puede aplicar un ajuste manual de valor de mercado dentro del
  escenario para probar estres o caidas.
- Las variaciones de mercado deben tratarse como cambios de valorizacion, no como
  ingresos ni gastos normales.
- La conciliacion de una cuenta de inversion puede actualizar su balance sin
  crear transacciones de gasto normales.
- Las cuentas deben poder clasificarse por liquidez: liquida, con demora,
  restringida o no retirable.
- Cuentas tipo AFP deben excluirse por defecto del calculo de supervivencia si no
  son retirables.
- Productos voluntarios o potencialmente retirables, como APV u otros, pueden
  incluirse solo con supuestos explicitos de demora, impuestos o penalizaciones.
- Las cuentas no retirables pueden aparecer en patrimonio, pero no deben contar
  como runway disponible por defecto.
- El escenario debe mostrar saldos excluidos y la razon de exclusion.
- El usuario debe poder definir que categorias son esenciales dentro del
  escenario.
- El usuario debe poder ajustar manualmente montos mensuales por categoria
  esencial.
- El usuario debe poder reducir, eliminar o limitar categorias discrecionales en
  el escenario.
- Los ajustes del escenario no deben modificar transacciones reales ni reglas del
  presupuesto normal.
- El escenario debe mostrar que supuestos cambiaron respecto de la proyeccion
  base.
- Un escenario de despido debe poder estimar compensacion por termino laboral,
  finiquito u otros pagos de salida como ingreso proyectado.
- El calculo debe basarse en datos configurables como sueldo, antiguedad,
  contrato, vacaciones pendientes, aviso previo, topes, descuentos y reglas por
  pais cuando correspondan.
- La compensacion estimada debe mostrarse como supuesto del escenario, no como
  ingreso real ni asesoría legal.
- Cuando el pago ocurra realmente, debe registrarse como transaccion real y puede
  vincularse con la estimacion del escenario.
- Debe mover automaticamente dinero reservado para objetivos hacia gastos
  esenciales si el escenario lo permite.
- Debe respetar reservas que el usuario marque como intocables.
- Las reservas u objetivos que pueden usarse como respaldo de emergencia deben
  estar habilitados explicitamente por el usuario.
- Las reglas de fallback deben tener prioridad u orden de uso.
- El escenario debe mostrar que reserva se usa primero, cuanto se consume y que
  objetivo queda afectado.
- Usar una reserva como fallback en un escenario no debe crear movimientos reales
  hasta que el usuario los ejecute.
- Si dos reservas tienen la misma prioridad y no existe regla de desempate, el
  sistema debe mostrar la ambiguedad.
- Los pagos minimos obligatorios de deudas deben continuar en el escenario de
  supervivencia por defecto.
- El usuario puede elegir por deuda si quiere modelar pago minimo, pago total u
  otra estrategia definida dentro del escenario.
- La estrategia elegida debe afectar el runway proyectado.
- El escenario debe separar gastos esenciales de pagos obligatorios de deuda.
- El usuario puede crear supuestos para reducir, pausar, refinanciar o incumplir
  pagos de deuda dentro del escenario.
- Esos supuestos deben quedar claramente marcados y no modificar los registros
  reales de deuda.
- Pagos omitidos o atrasados pueden modelarse como consecuencias del escenario,
  pero no deben crear transacciones reales mientras no ocurran.
- Debe distinguir gastos esenciales de gastos discrecionales.
- Debe permitir ajustar el gasto mensual minimo manualmente.

### UC-018: Revisar y resolver transacciones duplicadas

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
- Las sugerencias de duplicados deben tener nivel de confianza, como alta, media
  o baja.
- La confianza debe considerar monto, fecha, moneda, cuenta, origen, descripcion,
  contraparte, identificadores externos y registros vinculados.
- El sistema no debe eliminar duplicados automaticamente aunque la confianza sea
  alta.
- El usuario debe poder marcar una sugerencia como "no volver a sugerir".
- Una sugerencia descartada no debe reaparecer salvo que cambien datos relevantes
  de los movimientos involucrados.
- Debe distinguir duplicados reales de pagos repetidos validos.
- Debe distinguir entre duplicados del mismo movimiento real y registros
  complementarios, como Splitwise, que explican reparto, deuda o reembolso.
- Debe soportar duplicados entre transacciones manuales, importadas, Splitwise y
  archivos.
- Debe registrar que fuentes externas quedaron asociadas a la transaccion final.
- La revision de duplicados debe permitir resolver casos donde faltan, cambiaron
  o se reutilizaron identificadores externos.
- Debe permitir deshacer una fusion cuando sea necesario.

### UC-019: Conciliar cuentas

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
- En cuentas de efectivo, una diferencia de conciliacion puede clasificarse como
  perdida desconocida, gasto olvidado, regalo, redondeo, correccion u otra razon.
- El usuario debe poder agregar un comentario a la diferencia de efectivo.
- Si la diferencia corresponde a un gasto real conocido, el usuario puede
  asociarla a una categoria de gasto.
- Si la causa no es clara, el ajuste debe quedar visible como ajuste de
  conciliacion y no como gasto normal.
- Las cuentas manuales pueden quedar con saldo negativo.
- Una cuenta manual puede configurarse para permitir saldo negativo cuando
  representa linea de credito, deuda informal u otro caso valido.
- Si una cuenta que no permite saldo negativo queda bajo cero, el sistema debe
  advertirlo sin bloquear el registro.
- La advertencia debe ayudar a revisar transferencias faltantes, ingresos
  omitidos, gastos duplicados o diferencias de conciliacion.
- Los reportes deben tratar saldos negativos segun el tipo de cuenta: como deuda
  cuando corresponda o como alerta de revision en cuentas de activo.
- Debe funcionar para cuentas manuales, bancarias, efectivo y multimoneda.
- Debe ayudar a encontrar transacciones faltantes o duplicadas antes de crear un
  ajuste.

### UC-020: Preparar informacion para declaracion anual en Chile

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

### UC-021: Exponer API publica o MCP

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
Los reportes deben distinguir valores reales, pendientes, proyectados, ajustados
y conciliados. Cuando un total combine estados distintos, debe mostrar la
composicion o advertir que incluye valores no finales.
Los reportes deben permitir filtrar y agrupar por categoria, subcategoria y
etiquetas. Como las etiquetas pueden superponerse, los reportes por etiqueta
deben advertir cuando los totales no son mutuamente excluyentes.

### Reportes prioritarios para el PoC

- Salud financiera: muestra si el usuario esta ahorrando suficiente, si la deuda
  esta controlada, si hay fondo de emergencia y si los gastos son sostenibles.
  Debe partir con reglas predefinidas y permitir reglas configurables mas
  adelante. No debe depender de un puntaje exacto unico; debe mostrar estados por
  regla, confianza de los datos y supuestos usados.
- Periodo en curso contra proyeccion: compara gasto, ingreso, ahorro y reservas
  esperadas contra lo real mientras el periodo sigue abierto. Debe separar
  valores reales, pendientes y proyectados.
- Cierre de periodo contra proyeccion: muestra como termino el periodo contra lo
  proyectado, que categorias explican la diferencia y que supuestos fallaron.
  Debe distinguir valores conciliados, ajustes posteriores y valores estimados.
- Desglose de gasto: muestra en que se esta gastando dinero por categoria,
  subcategoria, cuenta, comercio, etiqueta, periodo y tipo de gasto. Debe
  permitir ver gasto bruto, reembolsos/devoluciones y gasto neto, sin asumir que
  todo ingreso es devolucion.
- Gastos compartidos: muestra gasto bruto pagado, parte del usuario, reembolsos
  pendientes, deudas con otras personas y gasto neto final. Mientras el
  reembolso no este pagado, el monto desembolsado debe considerarse salida del
  usuario y la parte por cobrar o pagar debe reflejarse como prestamo, deuda o
  saldo pendiente.
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
  inversiones y saldos multimoneda. Para v1 puede usar los balances disponibles
  reportados por instituciones o ingresados manualmente, pero debe permitir
  etiquetar activos como liquidos, con demora, restringidos, no retirables o
  volatiles para reportes futuros.
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
- Subcategoria: clasificacion dependiente de una categoria para analisis mas
  detallado.
- Etiqueta: clasificacion flexible adicional.
- Gasto compartido: gasto, deuda o reembolso asociado a otra persona o grupo.
- Prestamo o deuda entre personas: saldo pendiente asociado a un gasto compartido
  o movimiento personal hasta que se pague o cobre.
- Estimado: movimiento futuro esperado que aun no es transaccion real.
- Objetivo: meta financiera con monto, plazo y avance.
- Fondo total asignado a objetivos: lectura consolidada del dinero reservado para
  objetivos y su distribucion entre metas.
- Plan: politica de asignacion de dinero sobre una proyeccion.
- Escenario: combinacion de una proyeccion, sus estimados y uno o mas planes.
- Evento de termino laboral: supuesto de escenario que elimina o reduce ingresos
  y puede agregar compensaciones estimadas como finiquito.
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
