using Lukas.Application.Ledger;
using Lukas.Infrastructure.Ledger;
using Lukas.Infrastructure.Persistence;
using Microsoft.EntityFrameworkCore;
using Microsoft.EntityFrameworkCore.Diagnostics;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddControllers();
builder.Services.AddOpenApi();
builder.Services.AddCors(options =>
{
    options.AddPolicy(
        "FrontendDev",
        policy => policy
            .WithOrigins("http://localhost:5173", "http://127.0.0.1:5173", "tauri://localhost")
            .AllowAnyHeader()
            .AllowAnyMethod());
});
builder.Services.AddDbContext<LukasDbContext>(options =>
    options
        .UseSqlite(builder.Configuration.GetConnectionString("Lukas"))
        .ConfigureWarnings(warnings => warnings.Ignore(RelationalEventId.PendingModelChangesWarning)));
builder.Services.AddScoped<ILedgerRepository, EfLedgerRepository>();
builder.Services.AddScoped<LedgerService>();

var app = builder.Build();

if (app.Environment.IsDevelopment())
{
    app.MapOpenApi();
}

app.UseCors("FrontendDev");

app.MapGet("/health/live", () => Results.Ok(new { status = "live" }))
    .WithName("HealthLive");

app.MapGet("/health/ready", () => Results.Ok(new { status = "ready" }))
    .WithName("HealthReady");

app.MapControllers();

using (var scope = app.Services.CreateScope())
{
    var dbContext = scope.ServiceProvider.GetRequiredService<LukasDbContext>();
    dbContext.Database.Migrate();
}

app.Run();
