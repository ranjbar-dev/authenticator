using System.Text;

namespace Authenticator.Core.Import;

/// <summary>Minimal protobuf wire-format reader/writer, enough for Google Authenticator's MigrationPayload.</summary>
internal ref struct ProtoReader(ReadOnlySpan<byte> data)
{
    private readonly ReadOnlySpan<byte> _data = data;
    private int _pos;

    public bool End => _pos >= _data.Length;

    public (int Field, int WireType) ReadTag()
    {
        var tag = ReadVarint();
        return ((int)(tag >> 3), (int)(tag & 7));
    }

    public ulong ReadVarint()
    {
        ulong result = 0;
        for (var shift = 0; shift < 64; shift += 7)
        {
            if (_pos >= _data.Length)
                throw new OtpFormatException("The export QR code is truncated.");
            var b = _data[_pos++];
            result |= (ulong)(b & 0x7F) << shift;
            if ((b & 0x80) == 0)
                return result;
        }
        throw new OtpFormatException("The export QR code is malformed.");
    }

    public ReadOnlySpan<byte> ReadBytes()
    {
        var length = (int)ReadVarint();
        if (length < 0 || _pos + length > _data.Length)
            throw new OtpFormatException("The export QR code is truncated.");
        var slice = _data.Slice(_pos, length);
        _pos += length;
        return slice;
    }

    public string ReadString() => Encoding.UTF8.GetString(ReadBytes());

    public void Skip(int wireType)
    {
        switch (wireType)
        {
            case 0: ReadVarint(); break;
            case 1: _pos += 8; break;
            case 2: ReadBytes(); break;
            case 5: _pos += 4; break;
            default: throw new OtpFormatException("The export QR code is malformed.");
        }
    }
}

internal sealed class ProtoWriter
{
    private readonly MemoryStream _stream = new();

    public ProtoWriter Varint(int field, ulong value)
    {
        if (value == 0)
            return this;
        WriteRawVarint(((ulong)field << 3) | 0);
        WriteRawVarint(value);
        return this;
    }

    public ProtoWriter Bytes(int field, ReadOnlySpan<byte> value)
    {
        WriteRawVarint(((ulong)field << 3) | 2);
        WriteRawVarint((ulong)value.Length);
        _stream.Write(value);
        return this;
    }

    public ProtoWriter String(int field, string value) =>
        string.IsNullOrEmpty(value) ? this : Bytes(field, Encoding.UTF8.GetBytes(value));

    public byte[] ToArray() => _stream.ToArray();

    private void WriteRawVarint(ulong value)
    {
        while (value >= 0x80)
        {
            _stream.WriteByte((byte)(value | 0x80));
            value >>= 7;
        }
        _stream.WriteByte((byte)value);
    }
}
