using System.Globalization;

namespace Lukas.Application.Ledger;

public readonly record struct MonthlyPeriod
{
    public MonthlyPeriod(string key, DateOnly startDate)
    {
        Key = key;
        StartDate = startDate;
        EndDate = startDate.AddMonths(1);
    }

    public string Key { get; }

    public DateOnly StartDate { get; }

    public DateOnly EndDate { get; }

    public static bool TryParse(string? value, out MonthlyPeriod period)
    {
        period = default;
        if (value is null || !IsPeriodKey(value))
        {
            return false;
        }

        var key = value;
        if (!DateOnly.TryParseExact(
            $"{key}-01",
            "yyyy-MM-dd",
            CultureInfo.InvariantCulture,
            DateTimeStyles.None,
            out var startDate))
        {
            return false;
        }

        period = new MonthlyPeriod(key, startDate);
        return true;
    }

    private static bool IsPeriodKey(string value)
    {
        return value.Length == 7
            && value[4] == '-'
            && value[..4].All(static character => character is >= '0' and <= '9')
            && value[5..].All(static character => character is >= '0' and <= '9');
    }
}
