using System.Globalization;

namespace Lukas.Application.Ledger;

public sealed class LedgerService(ILedgerRepository repository)
{
    public Task<Result<AccountReadModel>> CreateAccountAsync(
        CreateAccountCommand command,
        CancellationToken cancellationToken)
    {
        if (string.IsNullOrWhiteSpace(command.DisplayName))
        {
            return Failure<AccountReadModel>("account.display_name_required", "Account display name is required.");
        }

        if (!IsAccountType(command.Type))
        {
            return Failure<AccountReadModel>("account.invalid_type", "Account type is invalid.");
        }

        if (!IsValidCurrency(command.Currency))
        {
            return Failure<AccountReadModel>("money.invalid_currency", "Currency must be an uppercase ISO 4217 code.");
        }

        if (command.OpeningBalance is not null && !TryParseAmount(command.OpeningBalance, out _))
        {
            return Failure<AccountReadModel>("money.invalid_amount", "Opening balance must be a decimal number.");
        }

        return Success(repository.CreateAccountAsync(command, cancellationToken));
    }

    public Task<IReadOnlyList<AccountReadModel>> GetAccountsAsync(CancellationToken cancellationToken)
    {
        return repository.GetAccountsAsync(cancellationToken);
    }

    public Task<Result<TransactionReadModel>> CreateManualTransactionAsync(
        CreateManualTransactionCommand command,
        CancellationToken cancellationToken)
    {
        if (!IsTransactionKind(command.Kind))
        {
            return Failure<TransactionReadModel>("transaction.invalid_kind", "Transaction kind is invalid.");
        }

        if (!TryParseAmount(command.Amount, out var amount) || amount <= 0)
        {
            return Failure<TransactionReadModel>("money.invalid_amount", "Transaction amount must be greater than zero.");
        }

        if (!IsValidCurrency(command.Currency))
        {
            return Failure<TransactionReadModel>("money.invalid_currency", "Currency must be an uppercase ISO 4217 code.");
        }

        if (!IsAccountEffect(command.AccountEffect))
        {
            return Failure<TransactionReadModel>("transaction.invalid_account_effect", "Account effect is invalid.");
        }

        if (string.IsNullOrWhiteSpace(command.Description))
        {
            return Failure<TransactionReadModel>("transaction.description_required", "Transaction description is required.");
        }

        return Success(repository.CreateManualTransactionAsync(command, cancellationToken));
    }

    public Task<Result<IReadOnlyList<TransactionReadModel>>> GetTransactionsAsync(
        string period,
        CancellationToken cancellationToken)
    {
        if (!IsPeriod(period))
        {
            return Failure<IReadOnlyList<TransactionReadModel>>("period.invalid", "Period must use YYYY-MM.");
        }

        return Success(repository.GetTransactionsAsync(period, cancellationToken));
    }

    public Task<Result<MonthlyCashflowReadModel>> GetMonthlyCashflowAsync(
        string period,
        CancellationToken cancellationToken)
    {
        if (!IsPeriod(period))
        {
            return Failure<MonthlyCashflowReadModel>("period.invalid", "Period must use YYYY-MM.");
        }

        return Success(repository.GetMonthlyCashflowAsync(period, cancellationToken));
    }

    private static async Task<Result<T>> Success<T>(Task<T> task)
    {
        try
        {
            return Result<T>.Success(await task.ConfigureAwait(false));
        }
        catch (LedgerRuleException exception)
        {
            return Result<T>.Failure(new LedgerError(exception.Code, exception.Message));
        }
    }

    private static Task<Result<T>> Failure<T>(string code, string message)
    {
        return Task.FromResult(Result<T>.Failure(new LedgerError(code, message)));
    }

    private static bool IsAccountType(string value)
    {
        return value is "cash" or "bank" or "credit_card";
    }

    private static bool IsTransactionKind(string value)
    {
        return value is "income" or "expense" or "adjustment";
    }

    private static bool IsAccountEffect(string value)
    {
        return value is "increase" or "decrease";
    }

    private static bool TryParseAmount(string value, out decimal amount)
    {
        return decimal.TryParse(value, NumberStyles.Number, CultureInfo.InvariantCulture, out amount);
    }

    private static bool IsValidCurrency(string currency)
    {
        return currency.Length == 3 && currency.All(static character => character is >= 'A' and <= 'Z');
    }

    private static bool IsPeriod(string value)
    {
        return value.Length == 7
            && value[4] == '-'
            && value[..4].All(static character => character is >= '0' and <= '9')
            && value[5..].All(static character => character is >= '0' and <= '9')
            && DateOnly.TryParseExact(
                $"{value}-01",
                "yyyy-MM-dd",
                CultureInfo.InvariantCulture,
                DateTimeStyles.None,
                out _);
    }
}

public sealed class LedgerRuleException(string code, string message) : Exception(message)
{
    public string Code { get; } = code;
}

public readonly record struct Result<T>
{
    private readonly T? value;
    private readonly LedgerError? error;

    private Result(T value)
    {
        this.value = value;
        error = null;
        IsSuccess = true;
    }

    private Result(LedgerError error)
    {
        value = default;
        this.error = error;
        IsSuccess = false;
    }

    public bool IsSuccess { get; }

    public bool IsFailure => !IsSuccess;

    public T Value => IsSuccess
        ? value!
        : throw new InvalidOperationException("Cannot access the value of a failed result.");

    public LedgerError Error => IsFailure
        ? error!
        : throw new InvalidOperationException("Cannot access the error of a successful result.");

    public static Result<T> Success(T value)
    {
        ArgumentNullException.ThrowIfNull(value);
        return new Result<T>(value);
    }

    public static Result<T> Failure(LedgerError error)
    {
        return new Result<T>(error);
    }
}

public sealed record LedgerError(string Code, string Message);
