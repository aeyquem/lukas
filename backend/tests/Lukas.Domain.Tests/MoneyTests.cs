using Shouldly;
using MoneyValue = Lukas.Domain.Money.Money;

namespace Lukas.Domain.Tests;

public sealed class MoneyTests
{
    [Theory]
    [InlineData("CLP")]
    [InlineData("USD")]
    public void UppercaseThreeLetterCurrencyIsValid(string currency)
    {
        MoneyValue.IsValidCurrency(currency).ShouldBeTrue();
    }

    [Theory]
    [InlineData("clp")]
    [InlineData("CL")]
    [InlineData("CLP1")]
    public void InvalidCurrencyIsRejected(string currency)
    {
        MoneyValue.IsValidCurrency(currency).ShouldBeFalse();
    }

    [Fact]
    public void AmountUsesInvariantDecimalParsing()
    {
        var parsed = MoneyValue.TryParseAmount("1234.56", out var amount);

        parsed.ShouldBeTrue();
        amount.ShouldBe(1234.56m);
        MoneyValue.FormatAmount(amount).ShouldBe("1234.56");
    }
}
