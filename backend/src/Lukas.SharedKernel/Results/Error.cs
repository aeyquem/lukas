namespace Lukas.SharedKernel.Results;

public sealed record Error(string Code, string Message)
{
    public static Error Unexpected(string message)
    {
        return new Error("unexpected", message);
    }
}
