using System.Globalization;

namespace Lukas.Application.Ledger;

public sealed class LedgerService(ILedgerRepository repository)
{
    private const int MaxTagsPerTransaction = 10;
    private const int MaxTagLength = 60;
    private const string HexColorPattern = "#0123456789ABCDEFabcdef";

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

    public Task<IReadOnlyList<TagReadModel>> GetTagsAsync(CancellationToken cancellationToken)
    {
        return repository.GetTagsAsync(cancellationToken);
    }

    public Task<Result<TagReadModel>> CreateTagAsync(
        CreateTagCommand command,
        CancellationToken cancellationToken)
    {
        var error = ValidateTag(command.Name, command.Color);
        if (error is not null)
        {
            return Failure<TagReadModel>(error.Code, error.Message);
        }

        return Success(repository.CreateTagAsync(command, cancellationToken));
    }

    public Task<Result<TagReadModel>> UpdateTagAsync(
        UpdateTagCommand command,
        CancellationToken cancellationToken)
    {
        var error = ValidateTag(command.Name, command.Color);
        if (error is not null)
        {
            return Failure<TagReadModel>(error.Code, error.Message);
        }

        return Success(repository.UpdateTagAsync(command, cancellationToken));
    }

    public async Task<Result<bool>> DeleteTagAsync(Guid id, CancellationToken cancellationToken)
    {
        try
        {
            await repository.DeleteTagAsync(id, cancellationToken).ConfigureAwait(false);
            return Result<bool>.Success(true);
        }
        catch (LedgerRuleException exception)
        {
            return Result<bool>.Failure(new LedgerError(exception.Code, exception.Message));
        }
    }

    public Task<Result<TransactionReadModel>> CreateManualTransactionAsync(
        CreateManualTransactionCommand command,
        CancellationToken cancellationToken)
    {
        var error = ValidateTransaction(
            command.Kind,
            command.AccountId,
            command.DestinationAccountId,
            command.Amount,
            command.Currency,
            command.AccountEffect,
            command.Description,
            command.Tags);
        if (error is not null)
        {
            return Failure<TransactionReadModel>(error.Code, error.Message);
        }

        return Success(repository.CreateManualTransactionAsync(command, cancellationToken));
    }

    public Task<Result<TransactionReadModel>> UpdateManualTransactionAsync(
        UpdateManualTransactionCommand command,
        CancellationToken cancellationToken)
    {
        var error = ValidateTransaction(
            command.Kind,
            command.AccountId,
            command.DestinationAccountId,
            command.Amount,
            command.Currency,
            command.AccountEffect,
            command.Description,
            command.Tags);
        if (error is not null)
        {
            return Failure<TransactionReadModel>(error.Code, error.Message);
        }

        return Success(repository.UpdateManualTransactionAsync(command, cancellationToken));
    }

    public Task<IReadOnlyList<MovementReadModel>> GetMovementsAsync(CancellationToken cancellationToken)
    {
        return repository.GetMovementsAsync(cancellationToken);
    }

    public Task<Result<BudgetTreeReadModel>> GetBudgetTreeAsync(
        string period,
        CancellationToken cancellationToken)
    {
        if (!MonthlyPeriod.TryParse(period, out var monthlyPeriod))
        {
            return Failure<BudgetTreeReadModel>("period.invalid", "Period must use YYYY-MM.");
        }

        return Success(repository.GetBudgetTreeAsync(monthlyPeriod, cancellationToken));
    }

    public Task<Result<BudgetTreeReadModel>> SaveBudgetTreeAsync(
        string period,
        SaveBudgetTreeCommand command,
        CancellationToken cancellationToken)
    {
        if (!MonthlyPeriod.TryParse(period, out var monthlyPeriod))
        {
            return Failure<BudgetTreeReadModel>("period.invalid", "Period must use YYYY-MM.");
        }

        var error = ValidateBudgetTree(command);
        if (error is not null)
        {
            return Failure<BudgetTreeReadModel>(error.Code, error.Message);
        }

        return Success(repository.SaveBudgetTreeAsync(monthlyPeriod, command, cancellationToken));
    }

    public Task<Result<IReadOnlyList<TransactionReadModel>>> GetTransactionsAsync(
        string period,
        CancellationToken cancellationToken)
    {
        if (!MonthlyPeriod.TryParse(period, out var monthlyPeriod))
        {
            return Failure<IReadOnlyList<TransactionReadModel>>("period.invalid", "Period must use YYYY-MM.");
        }

        return Success(repository.GetTransactionsAsync(monthlyPeriod, cancellationToken));
    }

    public Task<Result<MonthlyCashflowReadModel>> GetMonthlyCashflowAsync(
        string period,
        CancellationToken cancellationToken)
    {
        if (!MonthlyPeriod.TryParse(period, out var monthlyPeriod))
        {
            return Failure<MonthlyCashflowReadModel>("period.invalid", "Period must use YYYY-MM.");
        }

        return Success(repository.GetMonthlyCashflowAsync(monthlyPeriod, cancellationToken));
    }

    public Task<Result<MonthlyTagReportReadModel>> GetMonthlyTagReportAsync(
        string period,
        CancellationToken cancellationToken)
    {
        if (!MonthlyPeriod.TryParse(period, out var monthlyPeriod))
        {
            return Failure<MonthlyTagReportReadModel>("period.invalid", "Period must use YYYY-MM.");
        }

        return Success(repository.GetMonthlyTagReportAsync(monthlyPeriod, cancellationToken));
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
        return value is "income" or "expense" or "adjustment" or "transfer";
    }

    private static LedgerError? ValidateTransaction(
        string kind,
        Guid accountId,
        Guid? destinationAccountId,
        string amountValue,
        string currency,
        string accountEffect,
        string description,
        IReadOnlyList<string>? tags)
    {
        if (!IsTransactionKind(kind))
        {
            return new LedgerError("transaction.invalid_kind", "Transaction kind is invalid.");
        }

        if (!TryParseAmount(amountValue, out var amount) || amount <= 0)
        {
            return new LedgerError("money.invalid_amount", "Transaction amount must be greater than zero.");
        }

        if (!IsValidCurrency(currency))
        {
            return new LedgerError("money.invalid_currency", "Currency must be an uppercase ISO 4217 code.");
        }

        if (!IsAccountEffect(accountEffect))
        {
            return new LedgerError("transaction.invalid_account_effect", "Account effect is invalid.");
        }

        if (kind == "transfer" && destinationAccountId is null)
        {
            return new LedgerError("transaction.destination_account_required", "Destination account is required.");
        }

        if (kind == "transfer" && destinationAccountId == accountId)
        {
            return new LedgerError("transaction.same_transfer_account", "Transfer accounts must be different.");
        }

        if (kind == "transfer" && accountEffect != "decrease")
        {
            return new LedgerError("transaction.invalid_transfer_effect", "Transfer source account effect must be decrease.");
        }

        if (string.IsNullOrWhiteSpace(description))
        {
            return new LedgerError("transaction.description_required", "Transaction description is required.");
        }

        return ValidateTags(tags);
    }

    private static LedgerError? ValidateTags(IReadOnlyList<string>? tags)
    {
        if (tags is null)
        {
            return null;
        }

        if (tags.Count > MaxTagsPerTransaction)
        {
            return new LedgerError("tag.too_many", "A transaction cannot have more than 10 tags.");
        }

        if (tags.Any(string.IsNullOrWhiteSpace))
        {
            return new LedgerError("tag.name_required", "Tag names are required.");
        }

        if (tags.Any(tag => TransactionTagNames.DisplayName(tag).Length > MaxTagLength))
        {
            return new LedgerError("tag.name_too_long", "Tag names cannot be longer than 60 characters.");
        }

        if (tags.Select(TransactionTagNames.Normalize).ToHashSet().Count != tags.Count)
        {
            return new LedgerError("tag.duplicate", "Transaction tags must be unique.");
        }

        return null;
    }

    private static LedgerError? ValidateTag(string name, string color)
    {
        if (string.IsNullOrWhiteSpace(name))
        {
            return new LedgerError("tag.name_required", "Tag name is required.");
        }

        if (TransactionTagNames.DisplayName(name).Length > MaxTagLength)
        {
            return new LedgerError("tag.name_too_long", "Tag names cannot be longer than 60 characters.");
        }

        if (!IsHexColor(color))
        {
            return new LedgerError("tag.invalid_color", "Tag color must use #RRGGBB.");
        }

        return null;
    }

    private static bool IsHexColor(string value)
    {
        return value.Length == 7
            && value[0] == '#'
            && value.Skip(1).All(character => HexColorPattern.Contains(character));
    }

    private static LedgerError? ValidateBudgetTree(SaveBudgetTreeCommand command)
    {
        if (command.Categories.Count == 0)
        {
            return new LedgerError("budget.category_required", "At least one budget category is required.");
        }

        if (command.Subcategories.Count == 0)
        {
            return new LedgerError("budget.subcategory_required", "At least one budget subcategory is required.");
        }

        if (HasBlankName(command.Categories.Select(category => category.Name))
            || HasBlankName(command.Subcategories.Select(subcategory => subcategory.Name))
            || HasBlankName(command.Envelopes.Select(envelope => envelope.Name)))
        {
            return new LedgerError("budget.name_required", "Budget names are required.");
        }

        if (command.Envelopes.Any(envelope => !TryParseAmount(envelope.AssignedAmount, out _)))
        {
            return new LedgerError("money.invalid_amount", "Envelope assigned amounts must be decimal numbers.");
        }

        if (command.Goals.Any(goal => !TryParseAmount(goal.TargetAmount, out var targetAmount) || targetAmount <= 0))
        {
            return new LedgerError("money.invalid_amount", "Goal target amounts must be greater than zero.");
        }

        if (command.Goals.Any(goal => !IsGoalDeadline(goal.DeadlineDate)))
        {
            return new LedgerError("goal.invalid_deadline", "Goal deadlines must be blank or use YYYY-MM-DD.");
        }

        if (command.Goals.Any(goal => !IsGoalStatus(goal.Status)))
        {
            return new LedgerError("goal.invalid_status", "Goal status is invalid.");
        }

        var categoryIds = command.Categories.Select(category => category.Id).ToHashSet();
        var subcategoryIds = command.Subcategories.Select(subcategory => subcategory.Id).ToHashSet();
        var envelopeIds = command.Envelopes.Select(envelope => envelope.Id).ToHashSet();

        if (categoryIds.Count != command.Categories.Count
            || subcategoryIds.Count != command.Subcategories.Count
            || envelopeIds.Count != command.Envelopes.Count
            || command.Goals.Select(goal => goal.Id).ToHashSet().Count != command.Goals.Count)
        {
            return new LedgerError("budget.duplicate_id", "Budget node ids must be unique.");
        }

        if (command.Subcategories.Any(subcategory => !categoryIds.Contains(subcategory.CategoryId))
            || command.Envelopes.Any(envelope => !subcategoryIds.Contains(envelope.SubcategoryId)))
        {
            return new LedgerError("budget.invalid_parent", "Budget nodes must reference an existing parent.");
        }

        if (command.Goals.Any(goal => !envelopeIds.Contains(goal.EnvelopeId)))
        {
            return new LedgerError("goal.invalid_envelope", "Goals must reference an existing envelope.");
        }

        if (command.Goals.Select(goal => goal.EnvelopeId).ToHashSet().Count != command.Goals.Count)
        {
            return new LedgerError("goal.duplicate_envelope", "Only one goal can be linked to an envelope.");
        }

        return null;
    }

    private static bool HasBlankName(IEnumerable<string> names)
    {
        return names.Any(string.IsNullOrWhiteSpace);
    }

    private static bool IsAccountEffect(string value)
    {
        return value is "increase" or "decrease";
    }

    private static bool IsGoalDeadline(string value)
    {
        return value == string.Empty || DateOnly.TryParseExact(value, "yyyy-MM-dd", out _);
    }

    private static bool IsGoalStatus(string value)
    {
        return value is "active" or "completed" or "archived";
    }

    private static bool TryParseAmount(string value, out decimal amount)
    {
        return decimal.TryParse(value, NumberStyles.Number, CultureInfo.InvariantCulture, out amount);
    }

    private static bool IsValidCurrency(string currency)
    {
        return currency.Length == 3 && currency.All(static character => character is >= 'A' and <= 'Z');
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
