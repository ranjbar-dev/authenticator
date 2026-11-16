using System.Buffers.Binary;
using System.Net.Sockets;

namespace Authenticator.Infra.Platform;

/// <summary>Opt-in SNTP query to detect a wrong system clock (TOTP codes depend on it). The only network use.</summary>
public static class ClockCheck
{
    public const double WarningThresholdSeconds = 15;

    /// <summary>Returns how far the local clock is ahead (+) or behind (−) the server.</summary>
    public static async Task<TimeSpan> MeasureDriftAsync(string server = "time.windows.com", CancellationToken cancellationToken = default)
    {
        using var timeout = CancellationTokenSource.CreateLinkedTokenSource(cancellationToken);
        timeout.CancelAfter(TimeSpan.FromSeconds(4));

        using var udp = new UdpClient();
        udp.Connect(server, 123);

        var request = new byte[48];
        request[0] = 0x1B; // LI = 0, VN = 3, Mode = 3 (client)
        var sent = DateTimeOffset.UtcNow;
        await udp.SendAsync(request, timeout.Token);
        var response = await udp.ReceiveAsync(timeout.Token);
        var received = DateTimeOffset.UtcNow;

        if (response.Buffer.Length < 48)
            throw new InvalidDataException("Invalid response from the time server.");

        var seconds = BinaryPrimitives.ReadUInt32BigEndian(response.Buffer.AsSpan(40));
        var fraction = BinaryPrimitives.ReadUInt32BigEndian(response.Buffer.AsSpan(44));
        var serverTime = new DateTimeOffset(1900, 1, 1, 0, 0, 0, TimeSpan.Zero)
            .AddSeconds(seconds)
            .AddTicks((long)(fraction / (double)uint.MaxValue * TimeSpan.TicksPerSecond));

        var localMidpoint = sent + (received - sent) / 2;
        return localMidpoint - serverTime;
    }
}
