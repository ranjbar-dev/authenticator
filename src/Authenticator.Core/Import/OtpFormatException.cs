namespace Authenticator.Core.Import;

/// <summary>Raised when a QR payload or URI is not a usable authenticator account. The message is user-facing.</summary>
public sealed class OtpFormatException(string message) : Exception(message);
