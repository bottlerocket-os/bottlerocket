# \DefaultApi

All URIs are relative to *http://localhost:4242*

Method | HTTP request | Description
------------- | ------------- | -------------
[**commit_settings**](DefaultApi.md#commit_settings) | **post** /settings/commit | Commit pending settings
[**get_affected_services**](DefaultApi.md#get_affected_services) | **get** /metadata/affected-services | Get affected services
[**get_config_files**](DefaultApi.md#get_config_files) | **get** /configuration-files | Get configuration file data
[**get_pending_settings**](DefaultApi.md#get_pending_settings) | **get** /settings/pending | Get pending settings
[**get_services**](DefaultApi.md#get_services) | **get** /services | Get service data
[**get_settings**](DefaultApi.md#get_settings) | **get** /settings | Get current settings
[**set_settings**](DefaultApi.md#set_settings) | **patch** /settings | Update settings



## commit_settings

> commit_settings()
Commit pending settings

### Required Parameters

This endpoint does not need any parameter.

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_affected_services

> ::std::collections::HashMap<String, Vec<String>> get_affected_services(keys)
Get affected services

### Required Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **keys** | [**Vec<String>**](String.md)| Specific keys to query | 

### Return type

[**::std::collections::HashMap<String, Vec<String>>**](array.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_config_files

> model::ConfigurationFiles get_config_files(optional)
Get configuration file data

### Required Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters

Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **names** | [**Vec<String>**](String.md)| Specific configuration files to query | 

### Return type

[**model::ConfigurationFiles**](model::ConfigurationFiles.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_pending_settings

> model::Settings get_pending_settings()
Get pending settings

### Required Parameters

This endpoint does not need any parameter.

### Return type

[**model::Settings**](model::Settings.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_services

> model::Services get_services(optional)
Get service data

### Required Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters

Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **names** | [**Vec<String>**](String.md)| Specific services to query | 

### Return type

[**model::Services**](model::Services.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## get_settings

> model::Settings get_settings(optional)
Get current settings

### Required Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **optional** | **map[string]interface{}** | optional parameters | nil if no parameters

### Optional Parameters

Optional parameters are passed through a map[string]interface{}.

Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
 **keys** | [**Vec<String>**](String.md)| Specific keys to query. Takes precedence over 'prefix' if both query parameters are supplied | 
 **prefix** | **String**| Specific key prefix to query. This parameter will be ignored if 'keys' is also supplied | 

### Return type

[**model::Settings**](model::Settings.md)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: Not defined
- **Accept**: application/json

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)


## set_settings

> set_settings(body)
Update settings

### Required Parameters


Name | Type | Description  | Notes
------------- | ------------- | ------------- | -------------
  **body** | **model::Settings**|  | 

### Return type

 (empty response body)

### Authorization

No authorization required

### HTTP request headers

- **Content-Type**: application/json
- **Accept**: Not defined

[[Back to top]](#) [[Back to API list]](../README.md#documentation-for-api-endpoints) [[Back to Model list]](../README.md#documentation-for-models) [[Back to README]](../README.md)

