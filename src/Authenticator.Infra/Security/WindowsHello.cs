using System.Runtime.InteropServices;
using Windows.Security.Credentials;
using Windows.Security.Cryptography;

namespace Authenticator.Infra.Security;

public class WindowsHelloException(string message) : Exception(message);

/// <summary>The Hello key was removed (e.g. Hello reset). The vault can only be recovered from a backup.</summary>
public sealed class WindowsHelloKeyMissingException() : WindowsHelloException(
    "The Windows Hello key for Authenticator no longer exists. Restore your accounts from a backup file.");

/// <summary>
/// Windows Hello via <see cref="KeyCredentialManager"/>. The Hello private key never leaves the TPM; we have it
/// sign a fixed challenge and derive the vault wrapping key from the (deterministic RSA PKCS#1) signature, so the
/// vault cannot be decrypted without a successful Hello prompt.
/// </summary>
public static partial class WindowsHello
{
    private const string KeyName = "Authenticator.VaultKey";

    public static async Task<bool> IsAvailableAsync()
    {
        try
        {
            return await KeyCredentialManager.IsSupportedAsync();
        }
        catch (COMException)
        {
            return false;
        }
    }

    /// <summary>Creates (or replaces) the Hello key, then signs <paramref name="challenge"/>.</summary>
    public static async Task<byte[]> CreateAndSignAsync(byte[] challenge)
    {
        using var focus = BringPromptToFront();
        var result = await KeyCredentialManager.RequestCreateAsync(KeyName, KeyCredentialCreationOption.ReplaceExisting);
        ThrowIfFailed(result.Status);
        return await SignAsync(result.Credential, challenge);
    }

    public static async Task<byte[]> SignAsync(byte[] challenge)
    {
        var result = await KeyCredentialManager.OpenAsync(KeyName);
        ThrowIfFailed(result.Status);
        using var focus = BringPromptToFront();
        return await SignAsync(result.Credential, challenge);
    }

    public static async Task DeleteAsync()
    {
        try
        {
            await KeyCredentialManager.DeleteAsync(KeyName);
        }
        catch (Exception ex) when (ex is COMException or FileNotFoundException)
        {
            // Already gone.
        }
    }

    private static async Task<byte[]> SignAsync(KeyCredential credential, byte[] challenge)
    {
        var result = await credential.RequestSignAsync(CryptographicBuffer.CreateFromByteArray(challenge));
        ThrowIfFailed(result.Status);
        CryptographicBuffer.CopyToByteArray(result.Result, out var signature);
        return signature;
    }

    private static void ThrowIfFailed(KeyCredentialStatus status)
    {
        switch (status)
        {
            case KeyCredentialStatus.Success:
                return;
            case KeyCredentialStatus.UserCanceled:
                throw new OperationCanceledException();
            case KeyCredentialStatus.NotFound:
                throw new WindowsHelloKeyMissingException();
            case KeyCredentialStatus.UserPrefersPassword:
                throw new WindowsHelloException("Windows Hello was dismissed. Try again.");
            case KeyCredentialStatus.SecurityDeviceLocked:
                throw new WindowsHelloException("Windows Hello is locked after too many attempts. Sign in to Windows again, then retry.");
            default:
                throw new WindowsHelloException($"Windows Hello failed ({status}).");
        }
    }

    /// <summary>
    /// Desktop apps can't parent the Hello prompt, so it sometimes opens behind our window. Watch for it briefly
    /// and bring it to the foreground.
    /// </summary>
    private static IDisposable BringPromptToFront()
    {
        var cts = new CancellationTokenSource();
        _ = Task.Run(async () =>
        {
            for (var i = 0; i < 40 && !cts.IsCancellationRequested; i++)
            {
                var hwnd = FindWindowW("Credential Dialog Xaml Host", null);
                if (hwnd != 0)
                {
                    SetForegroundWindow(hwnd);
                    return;
                }
                await Task.Delay(100);
            }
        });
        return new CancelOnDispose(cts);
    }

    private sealed class CancelOnDispose(CancellationTokenSource cts) : IDisposable
    {
        public void Dispose() => cts.Cancel();
    }

    [LibraryImport("user32.dll", StringMarshalling = StringMarshalling.Utf16)]
    private static partial nint FindWindowW(string? className, string? windowName);

    [LibraryImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static partial bool SetForegroundWindow(nint hwnd);
}
