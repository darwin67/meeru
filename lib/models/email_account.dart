class EmailAccount {
  final String id;
  final String name;
  final String email;
  final String displayName;
  final EmailProvider provider;
  final ServerConfig imapConfig;
  final ServerConfig smtpConfig;
  final DateTime createdAt;
  final DateTime? lastSyncAt;
  final bool isActive;

  const EmailAccount({
    required this.id,
    required this.name,
    required this.email,
    required this.displayName,
    required this.provider,
    required this.imapConfig,
    required this.smtpConfig,
    required this.createdAt,
    this.lastSyncAt,
    this.isActive = true,
  });

  EmailAccount copyWith({
    String? id,
    String? name,
    String? email,
    String? displayName,
    EmailProvider? provider,
    ServerConfig? imapConfig,
    ServerConfig? smtpConfig,
    DateTime? createdAt,
    DateTime? lastSyncAt,
    bool? isActive,
  }) {
    return EmailAccount(
      id: id ?? this.id,
      name: name ?? this.name,
      email: email ?? this.email,
      displayName: displayName ?? this.displayName,
      provider: provider ?? this.provider,
      imapConfig: imapConfig ?? this.imapConfig,
      smtpConfig: smtpConfig ?? this.smtpConfig,
      createdAt: createdAt ?? this.createdAt,
      lastSyncAt: lastSyncAt ?? this.lastSyncAt,
      isActive: isActive ?? this.isActive,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'id': id,
      'name': name,
      'email': email,
      'displayName': displayName,
      'provider': provider.name,
      'imapConfig': imapConfig.toJson(),
      'smtpConfig': smtpConfig.toJson(),
      'createdAt': createdAt.toIso8601String(),
      'lastSyncAt': lastSyncAt?.toIso8601String(),
      'isActive': isActive,
    };
  }

  factory EmailAccount.fromJson(Map<String, dynamic> json) {
    return EmailAccount(
      id: json['id'],
      name: json['name'],
      email: json['email'],
      displayName: json['displayName'],
      provider: EmailProvider.values.byName(json['provider']),
      imapConfig: ServerConfig.fromJson(json['imapConfig']),
      smtpConfig: ServerConfig.fromJson(json['smtpConfig']),
      createdAt: DateTime.parse(json['createdAt']),
      lastSyncAt: json['lastSyncAt'] != null
          ? DateTime.parse(json['lastSyncAt'])
          : null,
      isActive: json['isActive'] ?? true,
    );
  }
}

class ServerConfig {
  final String host;
  final int port;
  final bool useSSL;
  final bool useTLS;
  final AuthMethod authMethod;

  const ServerConfig({
    required this.host,
    required this.port,
    required this.useSSL,
    required this.useTLS,
    required this.authMethod,
  });

  ServerConfig copyWith({
    String? host,
    int? port,
    bool? useSSL,
    bool? useTLS,
    AuthMethod? authMethod,
  }) {
    return ServerConfig(
      host: host ?? this.host,
      port: port ?? this.port,
      useSSL: useSSL ?? this.useSSL,
      useTLS: useTLS ?? this.useTLS,
      authMethod: authMethod ?? this.authMethod,
    );
  }

  Map<String, dynamic> toJson() {
    return {
      'host': host,
      'port': port,
      'useSSL': useSSL,
      'useTLS': useTLS,
      'authMethod': authMethod.name,
    };
  }

  factory ServerConfig.fromJson(Map<String, dynamic> json) {
    return ServerConfig(
      host: json['host'],
      port: json['port'],
      useSSL: json['useSSL'],
      useTLS: json['useTLS'],
      authMethod: AuthMethod.values.byName(json['authMethod']),
    );
  }
}

enum EmailProvider {
  gmail,
  outlook,
  yahoo,
  icloud,
  custom;

  String get displayName {
    switch (this) {
      case EmailProvider.gmail:
        return 'Gmail';
      case EmailProvider.outlook:
        return 'Outlook';
      case EmailProvider.yahoo:
        return 'Yahoo';
      case EmailProvider.icloud:
        return 'iCloud';
      case EmailProvider.custom:
        return 'Custom';
    }
  }
}

enum AuthMethod {
  password,
  oauth2,
  appPassword;

  String get displayName {
    switch (this) {
      case AuthMethod.password:
        return 'Password';
      case AuthMethod.oauth2:
        return 'OAuth 2.0';
      case AuthMethod.appPassword:
        return 'App Password';
    }
  }
}
