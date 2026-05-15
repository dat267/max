package config

import (
	"os"
	"path/filepath"
	"strings"

	"github.com/spf13/viper"
)

type Config struct {
	AppName string `mapstructure:"app_name"`
	Debug   bool   `mapstructure:"debug"`
}

func LoadConfig(cfgFile string) (*Config, error) {
	exePath, err := os.Executable()
	if err != nil {
		return nil, err
	}
	exeDir := filepath.Dir(exePath)

	if cfgFile != "" {
		viper.SetConfigFile(cfgFile)
	} else {
		viper.AddConfigPath(exeDir)
		viper.AddConfigPath("./configs") // Fallback
		viper.SetConfigName("config")
		viper.SetConfigType("yaml")
	}

	viper.SetEnvPrefix("MAX")
	viper.SetEnvKeyReplacer(strings.NewReplacer(".", "_"))
	viper.AutomaticEnv()

	viper.SetDefault("app_name", "MyDefaultApp")
	viper.SetDefault("debug", false)

	if err := viper.ReadInConfig(); err != nil {
		if _, ok := err.(viper.ConfigFileNotFoundError); ok {
			// Auto-create default config
			defaultCfg := filepath.Join(exeDir, "config.yaml")
			if err := viper.SafeWriteConfigAs(defaultCfg); err != nil {
				// Ignore if error is just config already exists, 
				// but log/handle others if necessary. 
				// Here we just proceed with defaults in memory.
			}
		} else {
			return nil, err
		}
	}

	var cfg Config
	if err := viper.Unmarshal(&cfg); err != nil {
		return nil, err
	}
	return &cfg, nil
}
