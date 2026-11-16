using System.Text.Json.Serialization;
using Authenticator.Core.Backup;
using Authenticator.Core.Models;
using Authenticator.Core.Vault;

namespace Authenticator.Core.Storage;

[JsonSourceGenerationOptions(
    PropertyNamingPolicy = JsonKnownNamingPolicy.CamelCase,
    UseStringEnumConverter = true,
    DefaultIgnoreCondition = JsonIgnoreCondition.WhenWritingNull)]
[JsonSerializable(typeof(VaultFile))]
[JsonSerializable(typeof(VaultContents))]
[JsonSerializable(typeof(BackupFile))]
[JsonSerializable(typeof(BackupContents))]
[JsonSerializable(typeof(List<Account>))]
public sealed partial class CoreJsonContext : JsonSerializerContext;
