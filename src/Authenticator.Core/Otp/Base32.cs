using System.Text;

namespace Authenticator.Core.Otp;

/// <summary>RFC 4648 Base32. Decoding is lenient: case-insensitive, ignores spaces, hyphens and padding.</summary>
public static class Base32
{
    private const string Alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";

    public static bool TryDecode(string? input, out byte[] bytes)
    {
        bytes = [];
        if (string.IsNullOrWhiteSpace(input))
            return false;

        var output = new List<byte>(input.Length * 5 / 8);
        int buffer = 0, bits = 0, symbols = 0;
        foreach (var raw in input)
        {
            if (raw is ' ' or '-' or '=' or '\t' or '\r' or '\n')
                continue;
            var index = Alphabet.IndexOf(char.ToUpperInvariant(raw));
            if (index < 0)
                return false;

            symbols++;
            buffer = (buffer << 5) | index;
            bits += 5;
            if (bits >= 8)
            {
                bits -= 8;
                output.Add((byte)(buffer >> bits));
                buffer &= (1 << bits) - 1;
            }
        }

        if (symbols == 0 || output.Count == 0)
            return false;
        bytes = output.ToArray();
        return true;
    }

    public static byte[] Decode(string input) =>
        TryDecode(input, out var bytes) ? bytes : throw new FormatException("Secret key is not valid Base32.");

    public static string Encode(ReadOnlySpan<byte> data)
    {
        var sb = new StringBuilder((data.Length * 8 + 4) / 5);
        int buffer = 0, bits = 0;
        foreach (var b in data)
        {
            buffer = (buffer << 8) | b;
            bits += 8;
            while (bits >= 5)
            {
                bits -= 5;
                sb.Append(Alphabet[(buffer >> bits) & 31]);
            }
        }
        if (bits > 0)
            sb.Append(Alphabet[(buffer << (5 - bits)) & 31]);
        return sb.ToString();
    }

    /// <summary>Groups an encoded key in blocks of four for readability.</summary>
    public static string Group(string encoded)
    {
        var sb = new StringBuilder();
        for (var i = 0; i < encoded.Length; i++)
        {
            if (i > 0 && i % 4 == 0)
                sb.Append(' ');
            sb.Append(encoded[i]);
        }
        return sb.ToString();
    }
}
