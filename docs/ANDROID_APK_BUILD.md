# Compilar o Starling Dev APK no nosso GitHub

A compilação usa GitHub Actions no repositório starling-mods e NÃO depende do servidor de builds do Starling.

- Fonte do jogo: Starling SHA 09fa16c167c4b2f8eb23fddd0faa808d56d0a2f2 (Minecraft 26.3).
- Mod compilado: BadOptimizations 0.1.0-alpha.2 (lightmap, partículas e FOV juntos).
- Ambiente: Android ARM64, Rust 1.95.0, JDK 25, NDK 28.2.13676358.
- Compilação: script oficial packaging/android/native_build.sh, PROFILE=quick e VARIANT=--dev.
- Artefato esperado: starling-dev-0.8.0-arm64-v8a.apk e build.json, no ZIP starling-dev-badoptimizations-arm64.
- O workflow verifica o SHA-256 da biblioteca dentro do APK, o identificador io.github.juanluci.starling.dev e a assinatura.

A branch feat/build-starling-dev-apk dispara o workflow automaticamente quando os arquivos de build mudam. Depois que a configuração entrar na branch main, será possível disparar também pelo botão Run workflow em Actions.

Passos para instalar: abrir Actions no nosso repositório, entrar na execução Build Starling Dev APK with BadOptimizations que esteja marcada como sucesso, baixar e extrair o artefato starling-dev-badoptimizations-arm64, instalar o APK Dev. O Dev usa pasta própria e não sobrescreve a instalação Preview; é necessário configurar seus próprios dados do Minecraft.

AVISO: Esta é uma versão de desenvolvimento para validar o funcionamento do mod. O perfil quick é adequado ao teste de execução; para benchmark comparável serão necessários builds stock e mod com perfil idêntico. O mod tem permissões nativas completas dentro do aplicativo. Não instale um artefato de fonte não confiável.
