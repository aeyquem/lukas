using System.Globalization;

namespace Lukas.Domain.Money;

public sealed record Money(decimal Amount, string Currency)
{
    public static Money Create(decimal amount, string currency)
    {
        if (!IsValidCurrency(currency))
        {
            throw new ArgumentException("Currency must be an uppercase ISO 4217 code.", nameof(currency));
        }

        return new Money(amount, currency);
    }

    public static bool TryParseAmount(string value, out decimal amount)
    {
        return decimal.TryParse(value, NumberStyles.Number, CultureInfo.InvariantCulture, out amount);
    }

    public static string FormatAmount(decimal amount)
    {
        return amount.ToString("0.#############################", CultureInfo.InvariantCulture);
    }

    public static bool IsValidCurrency(string currency)
    {
        return currency.Length == 3 && currency.All(static character => character is >= 'A' and <= 'Z');
    }
}
