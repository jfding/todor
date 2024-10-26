class TodorBin < Formula
  version 'v1.10.0'
  desc "todor - yet another cli TODO in Rust"
  homepage "https://github.com/jfding/todor"

  on_macos do
    on_intel do
      url "https://github.com/jfding/todor/releases/download/#{version}/todor-x86_64-apple-darwin.zip"
      sha256 "69aaff12164c7a0076836f8de6feac445232e48919a5f1c4272d31e042dc5b2e"
    end
    on_arm do
      url "https://github.com/jfding/todor/releases/download/#{version}/todor-aarch64-apple-darwin.zip"
      sha256 "a8713e804cc2f190119dd99a482af9254a088613ed1ea0ff55655ba7e98acf0d"
    end
  end
  on_linux do
    on_intel do
      url "https://github.com/jfding/todor/releases/download/#{version}/todor-x86_64-unknown-linux-musl.zip"
      sha256 "7848dc9d17e644f7496a9232d589628e0e4cf8f506dc8927a5404986816ecf34"
    end
    on_arm do
      url "https://github.com/jfding/todor/releases/download/#{version}/todor-aarch64-unknown-linux-musl.zip"
      sha256 "6df8410f2e7a39716a0e5397ca649341628459957b38d6f53379368790ba5be4"
    end
  end

  def install
    bin.install "todor"
  end
end
