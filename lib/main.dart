import 'package:flutter/material.dart' hide ThemeData;
import 'package:shadcn_ui/shadcn_ui.dart';
import 'package:provider/provider.dart';

import 'providers/auth_provider.dart';
import 'screens/welcome_screen.dart';

void main() {
  runApp(const MyApp());
}

class MyApp extends StatelessWidget {
  const MyApp({super.key});

  @override
  Widget build(BuildContext context) {
    return MultiProvider(
      providers: [
        ChangeNotifierProvider(create: (_) => AuthProvider()),
      ],
      child: ShadApp(
        title: 'Meeru - Email Client',
        theme: ShadThemeData(
          brightness: Brightness.light,
          colorScheme: const ShadSlateColorScheme.light(),
        ),
        darkTheme: ShadThemeData(
          brightness: Brightness.dark,
          colorScheme: const ShadSlateColorScheme.dark(),
        ),
        home: const AppRoot(),
      ),
    );
  }
}

class AppRoot extends StatefulWidget {
  const AppRoot({super.key});

  @override
  State<AppRoot> createState() => _AppRootState();
}

class _AppRootState extends State<AppRoot> {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addPostFrameCallback((_) {
      Provider.of<AuthProvider>(context, listen: false).loadAccounts();
    });
  }

  @override
  Widget build(BuildContext context) {
    return Consumer<AuthProvider>(
      builder: (context, authProvider, child) {
        if (authProvider.isLoading) {
          return Scaffold(
            backgroundColor: ShadTheme.of(context).colorScheme.background,
            body: const Center(
              child: CircularProgressIndicator(),
            ),
          );
        }

        if (authProvider.hasAccounts) {
          // TODO: Navigate to main email interface
          return Scaffold(
            backgroundColor: ShadTheme.of(context).colorScheme.background,
            body: Center(
              child: Column(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const Icon(Icons.check_circle, size: 64, color: Colors.green),
                  const SizedBox(height: 16),
                  Text(
                    'Account Setup Complete!',
                    style: ShadTheme.of(context).textTheme.h2,
                  ),
                  const SizedBox(height: 8),
                  Text(
                    'You have ${authProvider.accounts.length} account(s) configured.',
                    style: ShadTheme.of(context).textTheme.p,
                  ),
                  const SizedBox(height: 24),
                  ShadButton(
                    onPressed: () => authProvider.loadAccounts(),
                    child: const Text('Refresh'),
                  ),
                ],
              ),
            ),
          );
        }

        return const WelcomeScreen();
      },
    );
  }
}
