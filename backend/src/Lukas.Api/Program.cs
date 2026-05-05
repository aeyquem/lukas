var builder = WebApplication.CreateBuilder(args);

builder.Services.AddControllers();
builder.Services.AddOpenApi();

var app = builder.Build();

if (app.Environment.IsDevelopment())
{
    app.MapOpenApi();
}

app.MapGet("/health/live", () => Results.Ok(new { status = "live" }))
    .WithName("HealthLive");

app.MapGet("/health/ready", () => Results.Ok(new { status = "ready" }))
    .WithName("HealthReady");

app.MapControllers();

app.Run();
