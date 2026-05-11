using System.Globalization;

namespace Lukas.Application.Ledger;

public static class TransactionTagNames
{
    private static readonly char[] Separators = [' ', '\t', '\r', '\n', '-', '_'];

    public static string Normalize(string value)
    {
        return string.Join(
            " ",
            value
                .Split(Separators, StringSplitOptions.RemoveEmptyEntries | StringSplitOptions.TrimEntries)
                .Select(part => part.ToLowerInvariant()));
    }

    public static string DisplayName(string value)
    {
        var normalized = Normalize(value);
        return CultureInfo.InvariantCulture.TextInfo.ToTitleCase(normalized);
    }
}
