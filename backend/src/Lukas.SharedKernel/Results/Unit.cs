namespace Lukas.SharedKernel.Results;

public readonly record struct Unit
{
    public static Unit Value { get; } = new();
}
