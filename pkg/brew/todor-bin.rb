class TodorBin < Formula
  version 'v1.10.1'
  desc "todor - yet another cli TODO in Rust"
  homepage "https://github.com/jfding/todor"

  on_macos do
    on_intel do
      url "https://github.com/jfding/todor/releases/download/#{version}/todor-x86_64-apple-darwin.zip"
      sha256 "087b5df14322d2873f8f6268610bba4636cffd9db5f1ae1f8634db02db9c449b"

    end
    on_arm do
      url "https://github.com/jfding/todor/releases/download/#{version}/todor-aarch64-apple-darwin.zip"
      sha256 "c6d04438480dc0e85554ef502795db07cd519802f4f0700d30809d4b26239906"
    end
  end
  on_linux do
    on_intel do
      url "https://github.com/jfding/todor/releases/download/#{version}/todor-x86_64-unknown-linux-musl.zip"
      sha256 "8946b71bea92ae26de35804ce3773551fd3e6a9810dedf92dad58816a6ee87cd"
    end
    on_arm do
      url "https://github.com/jfding/todor/releases/download/#{version}/todor-aarch64-unknown-linux-musl.zip"
      sha256 "ba7b39a5ca327c27237f9e51e859fbf47b32cf5155ac01a5d7f0bbfb65a3aa2e"
    end
  end

  def install
    bin.install "todor"
    bin.install "today"
    bin.install "tomorrow"
    on_macos do
      bin.install "t2reminders.scpt"
    end
  end
end
