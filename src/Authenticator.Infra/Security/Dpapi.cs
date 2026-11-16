using System.Security.Cryptography;

namespace Authenticator.Infra.Security;

/// <summary>Windows DPAPI bound to the current user account.</summary>
internal static class Dpapi
{
    private static readonly byte[] Entropy = "Authenticator.Dpapi.v1"u8.ToArray();

    public static byte[] Protect(byte[] data) => ProtectedData.Protect(data, Entropy, DataProtectionScope.CurrentUser);

    public static byte[] Unprotect(byte[] data) => ProtectedData.Unprotect(data, Entropy, DataProtectionScope.CurrentUser);
}
