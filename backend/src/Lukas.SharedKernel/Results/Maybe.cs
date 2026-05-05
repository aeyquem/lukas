namespace Lukas.SharedKernel.Results;

public readonly record struct Maybe<T>
{
    private readonly T? value;

    private Maybe(T value)
    {
        this.value = value;
        HasValue = true;
    }

    public bool HasValue { get; }

    public bool IsNone => !HasValue;

    public T Value => HasValue
        ? value!
        : throw new InvalidOperationException("Cannot access the value of Maybe.None.");

    public static Maybe<T> Some(T value)
    {
        ArgumentNullException.ThrowIfNull(value);
        return new Maybe<T>(value);
    }

    public static Maybe<T> None()
    {
        return default;
    }
}
